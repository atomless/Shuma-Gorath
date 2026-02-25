// @ts-check

export const cloneJsonValue = (value) => {
  if (value === null || value === undefined) return null;
  if (typeof value !== 'object') return value;
  try {
    return JSON.parse(JSON.stringify(value));
  } catch (_e) {
    return null;
  }
};

export const readValueAtPath = (obj, path) => {
  const segments = String(path || '').split('.');
  let cursor = obj;
  for (const segment of segments) {
    if (!segment || cursor === null || typeof cursor !== 'object') return undefined;
    if (!Object.prototype.hasOwnProperty.call(cursor, segment)) return undefined;
    cursor = cursor[segment];
  }
  return cursor;
};

export const writeValueAtPath = (target, path, value) => {
  const segments = String(path || '').split('.');
  if (segments.length === 0) return;
  let cursor = target;
  for (let i = 0; i < segments.length; i += 1) {
    const segment = segments[i];
    if (!segment) return;
    const isLeaf = i === segments.length - 1;
    if (isLeaf) {
      cursor[segment] = value;
      return;
    }
    if (!cursor[segment] || typeof cursor[segment] !== 'object' || Array.isArray(cursor[segment])) {
      cursor[segment] = {};
    }
    cursor = cursor[segment];
  }
};

export const buildTemplateFromPaths = (source, paths = []) => {
  const template = {};
  paths.forEach((path) => {
    const rawValue = readValueAtPath(source, path);
    if (rawValue === undefined) return;
    const cloned = cloneJsonValue(rawValue);
    writeValueAtPath(template, path, cloned === null && rawValue !== null ? rawValue : cloned);
  });
  return template;
};

const JSON_LINE_COLUMN_RE = /line\s+(\d+)\s*column\s+(\d+)/i;
const JSON_POSITION_RE = /position\s+(\d+)/i;

const parsePositiveInteger = (value) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return null;
  return Math.floor(numeric);
};

export const lineColumnFromOffset = (raw, offset) => {
  const source = String(raw || '');
  const numericOffset = Number(offset);
  const boundedOffset = Number.isFinite(numericOffset)
    ? Math.max(0, Math.min(source.length, Math.floor(numericOffset)))
    : 0;

  let line = 1;
  let column = 1;
  for (let index = 0; index < boundedOffset; index += 1) {
    if (source[index] === '\n') {
      line += 1;
      column = 1;
    } else {
      column += 1;
    }
  }
  return { line, column };
};

const approximateJsonSyntaxLocation = (raw) => {
  const source = String(raw || '');
  if (!source) return { line: 1, column: 1 };

  let index = 0;
  let line = 1;
  let column = 1;

  const peek = () => (index < source.length ? source[index] : '');
  const atEnd = () => index >= source.length;

  const advance = () => {
    if (atEnd()) return '';
    const char = source[index];
    index += 1;
    if (char === '\n') {
      line += 1;
      column = 1;
    } else {
      column += 1;
    }
    return char;
  };

  const fail = () => ({ line, column });
  const isWhitespace = (char) =>
    char === ' ' || char === '\t' || char === '\r' || char === '\n';
  const isDigit = (char) => char >= '0' && char <= '9';
  const isHex = (char) =>
    (char >= '0' && char <= '9')
    || (char >= 'a' && char <= 'f')
    || (char >= 'A' && char <= 'F');

  const skipWhitespace = () => {
    while (!atEnd() && isWhitespace(peek())) advance();
  };

  const parseString = () => {
    if (peek() !== '"') throw fail();
    advance(); // opening quote
    while (!atEnd()) {
      const char = advance();
      if (char === '"') return;
      if (char === '\\') {
        if (atEnd()) throw fail();
        const escaped = advance();
        if (escaped === 'u') {
          for (let i = 0; i < 4; i += 1) {
            if (atEnd()) throw fail();
            if (!isHex(peek())) throw fail();
            advance();
          }
        } else if (!`"\\/bfnrt`.includes(escaped)) {
          throw fail();
        }
      } else if (char === '\n' || char === '\r' || char < ' ') {
        throw fail();
      }
    }
    throw fail();
  };

  const parseLiteral = (literal) => {
    for (let i = 0; i < literal.length; i += 1) {
      if (peek() !== literal[i]) throw fail();
      advance();
    }
  };

  const parseDigits = () => {
    let count = 0;
    while (!atEnd() && isDigit(peek())) {
      advance();
      count += 1;
    }
    return count;
  };

  const parseNumber = () => {
    if (peek() === '-') advance();
    if (peek() === '0') {
      advance();
    } else if (isDigit(peek())) {
      if (parseDigits() === 0) throw fail();
    } else {
      throw fail();
    }
    if (peek() === '.') {
      advance();
      if (parseDigits() === 0) throw fail();
    }
    if (peek() === 'e' || peek() === 'E') {
      advance();
      if (peek() === '+' || peek() === '-') advance();
      if (parseDigits() === 0) throw fail();
    }
  };

  const parseValue = () => {
    skipWhitespace();
    if (atEnd()) throw fail();
    const char = peek();
    if (char === '{') {
      parseObject();
      return;
    }
    if (char === '[') {
      parseArray();
      return;
    }
    if (char === '"') {
      parseString();
      return;
    }
    if (char === '-' || isDigit(char)) {
      parseNumber();
      return;
    }
    if (char === 't') {
      parseLiteral('true');
      return;
    }
    if (char === 'f') {
      parseLiteral('false');
      return;
    }
    if (char === 'n') {
      parseLiteral('null');
      return;
    }
    throw fail();
  };

  const parseArray = () => {
    if (peek() !== '[') throw fail();
    advance();
    skipWhitespace();
    if (peek() === ']') {
      advance();
      return;
    }
    while (true) {
      parseValue();
      skipWhitespace();
      if (peek() === ',') {
        advance();
        skipWhitespace();
        continue;
      }
      if (peek() === ']') {
        advance();
        return;
      }
      throw fail();
    }
  };

  const parseObject = () => {
    if (peek() !== '{') throw fail();
    advance();
    skipWhitespace();
    if (peek() === '}') {
      advance();
      return;
    }
    while (true) {
      parseString();
      skipWhitespace();
      if (peek() !== ':') throw fail();
      advance();
      parseValue();
      skipWhitespace();
      if (peek() === ',') {
        advance();
        skipWhitespace();
        continue;
      }
      if (peek() === '}') {
        advance();
        return;
      }
      throw fail();
    }
  };

  try {
    skipWhitespace();
    parseValue();
    skipWhitespace();
    if (!atEnd()) throw fail();
    return null;
  } catch (error) {
    const location = error && typeof error === 'object'
      ? /** @type {{ line?: number, column?: number }} */ (error)
      : {};
    const failureLine = parsePositiveInteger(location.line);
    const failureColumn = parsePositiveInteger(location.column);
    if (failureLine !== null && failureColumn !== null) {
      return { line: failureLine, column: failureColumn };
    }
    return { line, column };
  }
};

export const parseJsonSyntaxLocation = (raw, error) => {
  const message = error && error.message ? String(error.message) : '';
  const lineColumnMatch = message.match(JSON_LINE_COLUMN_RE);
  if (lineColumnMatch) {
    const line = parsePositiveInteger(lineColumnMatch[1]);
    const column = parsePositiveInteger(lineColumnMatch[2]);
    return { line, column };
  }

  const positionMatch = message.match(JSON_POSITION_RE);
  if (positionMatch) {
    const position = parsePositiveInteger(positionMatch[1]);
    if (position !== null) {
      return lineColumnFromOffset(raw, position);
    }
  }

  const approximate = approximateJsonSyntaxLocation(raw);
  if (approximate) return approximate;
  return { line: null, column: null };
};

export const parseJsonObjectWithDiagnostics = (raw) => {
  const source = String(raw || '{}');
  try {
    const parsed = JSON.parse(source);
    if (!parsed || Array.isArray(parsed) || typeof parsed !== 'object') {
      return {
        ok: false,
        normalized: null,
        value: null,
        issue: {
          field: '',
          message: 'Top-level JSON value must be an object.',
          expected: 'Use an object (for example: {"rate_limit": 120}).',
          received: parsed,
          line: 1,
          column: 1
        }
      };
    }
    return {
      ok: true,
      normalized: JSON.stringify(parsed),
      value: parsed,
      issue: null
    };
  } catch (error) {
    const location = parseJsonSyntaxLocation(source, error);
    const hasLine = Number.isInteger(location.line) && location.line > 0;
    const hasColumn = Number.isInteger(location.column) && location.column > 0;
    const lineColumnText = hasLine
      ? `line ${location.line}${hasColumn ? `, column ${location.column}` : ''}`
      : '';
    return {
      ok: false,
      normalized: null,
      value: null,
      issue: {
        field: '',
        message: lineColumnText
          ? `Invalid JSON syntax at ${lineColumnText}.`
          : 'Invalid JSON syntax.',
        expected: 'Fix JSON syntax (quotes, commas, and brackets).',
        received: error && error.message ? String(error.message) : 'Invalid JSON syntax.',
        line: hasLine ? location.line : null,
        column: hasColumn ? location.column : null
      }
    };
  }
};

export const buildJsonFieldLineMap = (raw) => {
  const source = String(raw || '');
  const lineMap = new Map();
  if (!source.trim()) return lineMap;

  let index = 0;
  let line = 1;

  const isWhitespace = (char) =>
    char === ' ' || char === '\t' || char === '\n' || char === '\r';

  const advance = (steps = 1) => {
    for (let offset = 0; offset < steps && index < source.length; offset += 1) {
      if (source[index] === '\n') line += 1;
      index += 1;
    }
  };

  const skipWhitespace = () => {
    while (index < source.length && isWhitespace(source[index])) {
      advance();
    }
  };

  const parseStringToken = () => {
    if (source[index] !== '"') return null;
    const startLine = line;
    advance(); // opening quote
    let value = '';
    while (index < source.length) {
      const char = source[index];
      if (char === '"') {
        advance(); // closing quote
        return { value, line: startLine };
      }
      if (char === '\\') {
        advance();
        if (index >= source.length) return null;
        const escaped = source[index];
        if (escaped === 'u') {
          const hex = source.slice(index + 1, index + 5);
          if (/^[0-9a-fA-F]{4}$/.test(hex)) {
            value += String.fromCharCode(parseInt(hex, 16));
            advance(5); // "u" + 4 hex chars
          } else {
            advance();
          }
          continue;
        }
        const escapeMap = {
          '"': '"',
          '\\': '\\',
          '/': '/',
          b: '\b',
          f: '\f',
          n: '\n',
          r: '\r',
          t: '\t'
        };
        value += Object.prototype.hasOwnProperty.call(escapeMap, escaped)
          ? escapeMap[escaped]
          : escaped;
        advance();
        continue;
      }
      value += char;
      advance();
    }
    return null;
  };

  const parsePrimitive = () => {
    while (index < source.length) {
      const char = source[index];
      if (
        char === ','
        || char === '}'
        || char === ']'
        || isWhitespace(char)
      ) {
        return;
      }
      advance();
    }
  };

  const parseValue = (path) => {
    skipWhitespace();
    if (index >= source.length) return;
    const char = source[index];
    if (char === '{') {
      parseObject(path);
      return;
    }
    if (char === '[') {
      parseArray(path);
      return;
    }
    if (char === '"') {
      parseStringToken();
      return;
    }
    parsePrimitive();
  };

  const parseArray = (path) => {
    if (source[index] !== '[') return;
    advance();
    skipWhitespace();
    if (source[index] === ']') {
      advance();
      return;
    }
    while (index < source.length) {
      parseValue(path);
      skipWhitespace();
      if (source[index] === ',') {
        advance();
        skipWhitespace();
        continue;
      }
      if (source[index] === ']') {
        advance();
        return;
      }
      return;
    }
  };

  const parseObject = (path) => {
    if (source[index] !== '{') return;
    advance();
    skipWhitespace();
    if (source[index] === '}') {
      advance();
      return;
    }
    while (index < source.length) {
      const keyToken = parseStringToken();
      if (!keyToken) return;
      const key = String(keyToken.value || '');
      const childPath = path ? `${path}.${key}` : key;
      if (key && !lineMap.has(childPath)) {
        lineMap.set(childPath, keyToken.line);
      }

      skipWhitespace();
      if (source[index] !== ':') return;
      advance();
      parseValue(childPath);
      skipWhitespace();
      if (source[index] === ',') {
        advance();
        skipWhitespace();
        continue;
      }
      if (source[index] === '}') {
        advance();
        return;
      }
      return;
    }
  };

  skipWhitespace();
  if (source[index] === '{') {
    parseObject('');
  }
  return lineMap;
};

export const resolveJsonFieldLine = (fieldPath, fieldLineMap) => {
  const path = String(fieldPath || '').trim();
  if (!path || !(fieldLineMap instanceof Map)) return null;
  if (fieldLineMap.has(path)) {
    const directLine = Number(fieldLineMap.get(path));
    return Number.isInteger(directLine) && directLine > 0 ? directLine : null;
  }

  const segments = path.split('.').filter(Boolean);
  const leaf = segments.length > 0 ? segments[segments.length - 1] : '';
  if (!leaf) return null;

  const matches = [];
  fieldLineMap.forEach((line, candidatePath) => {
    if (candidatePath === leaf || candidatePath.endsWith(`.${leaf}`)) {
      const parsedLine = Number(line);
      if (Number.isInteger(parsedLine) && parsedLine > 0) {
        matches.push(parsedLine);
      }
    }
  });
  if (matches.length === 1) return matches[0];
  return null;
};

export const normalizeJsonObjectForCompare = (raw) => {
  const result = parseJsonObjectWithDiagnostics(raw);
  return result.ok ? result.normalized : null;
};
