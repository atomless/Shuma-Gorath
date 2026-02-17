// @ts-check

import * as formUtils from './config-form-utils.js';
import * as domModule from './core/dom.js';

const domCache = domModule.createCache({ document });
const getById = domCache.byId;

  const OPTION_GROUP_KEYS = Object.freeze([
    'readers',
    'parsers',
    'updaters',
    'checks',
    'state',
    'actions',
    'callbacks'
  ]);
  const DRAFT_SETTER_ALIAS = Object.freeze({
    setMazeSavedState: 'maze',
    setHoneypotSavedState: 'honeypot',
    setBrowserPolicySavedState: 'browserPolicy',
    setBypassAllowlistSavedState: 'bypassAllowlists',
    setRobotsSavedState: 'robots',
    setAiPolicySavedState: 'aiPolicy',
    setPowSavedState: 'pow',
    setChallengePuzzleSavedState: 'challengePuzzle',
    setBotnessSavedState: 'botness',
    setCdpSavedState: 'cdp',
    setEdgeIntegrationModeSavedState: 'edgeMode',
    setRateLimitSavedState: 'rateLimit',
    setJsRequiredSavedState: 'jsRequired'
  });
  const GEO_DRAFT_FALLBACK = Object.freeze({
    risk: '',
    allow: '',
    challenge: '',
    maze: '',
    block: '',
    mutable: false
  });

  function flattenBindOptions(rawOptions = {}) {
    // Accept grouped option buckets to keep the bind callsite small while retaining
    // stable flat option names used throughout this module.
    const flattened = { ...rawOptions };
    OPTION_GROUP_KEYS.forEach((groupKey) => {
      const group = rawOptions[groupKey];
      if (!group || typeof group !== 'object') return;
      Object.entries(group).forEach(([key, value]) => {
        if (flattened[key] === undefined) {
          flattened[key] = value;
        }
      });
    });
    return flattened;
  }

  function normalizeContextOptions(rawOptions = {}) {
    if (rawOptions.domainApi && typeof rawOptions.domainApi === 'object') {
      const merged = { ...rawOptions, ...rawOptions.domainApi };
      delete merged.domainApi;
      return merged;
    }
    if (!rawOptions.context || typeof rawOptions.context !== 'object') return rawOptions;
    const context = rawOptions.context;
    const normalized = {
      statusPanel: context.statusPanel || null,
      apiClient: context.apiClient || null,
      effects:
        context.effects && typeof context.effects === 'object'
          ? context.effects
          : (rawOptions.effects && typeof rawOptions.effects === 'object' ? rawOptions.effects : null)
    };

    const auth = context.auth || {};
    const callbacks = context.callbacks || {};
    const readers = context.readers || {};
    const parsers = context.parsers || {};
    const updaters = context.updaters || {};
    const checks = context.checks || {};
    const actions = context.actions || {};

    if (typeof auth.getAdminContext === 'function') {
      normalized.getAdminContext = auth.getAdminContext;
    }
    if (typeof callbacks.onConfigSaved === 'function') {
      normalized.onConfigSaved = callbacks.onConfigSaved;
    }
    Object.assign(normalized, readers, parsers, updaters, checks, actions);

    const draft = context.draft || {};
    if (typeof draft.get === 'function') {
      normalized.getGeoSavedState = () => draft.get('geo', GEO_DRAFT_FALLBACK);
    }
    if (typeof draft.set === 'function') {
      Object.entries(DRAFT_SETTER_ALIAS).forEach(([setterName, sectionKey]) => {
        normalized[setterName] = (next) => draft.set(sectionKey, next);
      });
      normalized.setGeoSavedState = (next) => draft.set('geo', next);
    }

    return normalized;
  }

  function bind(rawOptions = {}) {
    const options = flattenBindOptions(normalizeContextOptions(rawOptions));
    const statusPanel = options.statusPanel || null;
    const applyStatusPatch =
      statusPanel && typeof statusPanel.applyPatch === 'function'
        ? statusPanel.applyPatch.bind(statusPanel)
        : (patch) => {
          if (!statusPanel) return;
          statusPanel.update(patch);
          statusPanel.render();
        };
    const apiClient = options.apiClient || null;
    const timerSetTimeout =
      options.effects && typeof options.effects.setTimer === 'function'
        ? options.effects.setTimer
        : window.setTimeout.bind(window);
    const requestImpl =
      options.effects && typeof options.effects.request === 'function'
        ? options.effects.request
        : fetch.bind(globalThis);
    const parseCountryCodesStrict = typeof options.parseCountryCodesStrict === 'function'
      ? options.parseCountryCodesStrict
      : formUtils.parseCountryCodesStrict;

    async function saveConfigPatch(messageTarget, patch) {
      let result;
      if (apiClient && typeof apiClient.updateConfig === 'function') {
        result = await apiClient.updateConfig(patch);
      } else {
        const ctx = options.getAdminContext(messageTarget || null);
        if (!ctx) {
          throw new Error('Missing admin API context');
        }
        const { endpoint, apikey, sessionAuth, csrfToken } = ctx;
        const headers = new Headers({
          'Content-Type': 'application/json',
          'Accept': 'application/json'
        });
        const apiKeyValue = String(apikey || '').trim();
        if (apiKeyValue) {
          headers.set('Authorization', `Bearer ${apiKeyValue}`);
        }
        if (sessionAuth === true && String(csrfToken || '').trim()) {
          headers.set('X-Shuma-CSRF', String(csrfToken).trim());
        }
        const resp = await requestImpl(`${endpoint}/admin/config`, {
          method: 'POST',
          headers,
          credentials: sessionAuth === true ? 'same-origin' : undefined,
          body: JSON.stringify(patch)
        });
        if (!resp.ok) {
          const text = await resp.text();
          throw new Error(text || 'Failed to save config');
        }
        result = await resp.json();
      }
      if (result && result.config && typeof result.config === 'object') {
        applyStatusPatch({ configSnapshot: result.config });
      }
      if (typeof options.onConfigSaved === 'function') {
        options.onConfigSaved(patch, result);
      }
      return result;
    }

    const setAdminMessage = (messageTarget, text, variant = 'info') => {
      if (!messageTarget) return;
      messageTarget.textContent = String(text || '');
      messageTarget.className = `message ${variant}`;
    };

    const beginSaveButton = (button, label = 'Saving...') => {
      if (!button) return;
      button.textContent = label;
      button.dataset.saving = 'true';
      button.disabled = true;
    };

    const endSaveButton = (button, label) => {
      if (!button) return;
      button.dataset.saving = 'false';
      if (label !== undefined) {
        button.textContent = String(label);
      }
    };

    const endSaveButtonAfter = (button, interimLabel, nextLabel, delayMs = 1500, onReset = null) => {
      if (!button) return;
      if (interimLabel !== undefined) {
        button.textContent = String(interimLabel);
      }
      timerSetTimeout(() => {
        endSaveButton(button, nextLabel);
        if (typeof onReset === 'function') onReset();
      }, delayMs);
    };

    function parseList(raw) {
      if (typeof options.parseListTextarea === 'function') {
        return options.parseListTextarea(raw);
      }
      return formUtils.parseListTextarea(raw);
    }

    function normalizeList(raw) {
      if (typeof options.normalizeListTextareaForCompare === 'function') {
        return options.normalizeListTextareaForCompare(raw);
      }
      return formUtils.normalizeListTextareaForCompare(raw);
    }

    function parseHoneypotPaths(raw) {
      if (typeof options.parseHoneypotPathsTextarea === 'function') {
        return options.parseHoneypotPathsTextarea(raw);
      }
      return formUtils.parseHoneypotPathsTextarea(raw);
    }

    function parseBrowserRules(raw) {
      if (typeof options.parseBrowserRulesTextarea === 'function') {
        return options.parseBrowserRulesTextarea(raw);
      }
      return formUtils.parseBrowserRulesTextarea(raw);
    }

    function normalizeBrowserRules(raw) {
      if (typeof options.normalizeBrowserRulesForCompare === 'function') {
        return options.normalizeBrowserRulesForCompare(raw);
      }
      return formUtils.normalizeBrowserRulesForCompare(raw);
    }

    const saveMazeButton = getById('save-maze-config');
    if (saveMazeButton) {
      saveMazeButton.onclick = async function saveMazeConfig() {
        const msg = getById('admin-msg');
        const btn = this;

        const mazeEnabled = getById('maze-enabled-toggle').checked;
        const mazeAutoBan = getById('maze-auto-ban-toggle').checked;
        const mazeThreshold = options.readIntegerFieldValue('maze-threshold', msg);
        if (mazeThreshold === null) return;

        beginSaveButton(btn);

        try {
          await saveConfigPatch(msg, {
            maze_enabled: mazeEnabled,
            maze_auto_ban: mazeAutoBan,
            maze_auto_ban_threshold: mazeThreshold
          });

          options.setMazeSavedState({
            enabled: mazeEnabled,
            autoBan: mazeAutoBan,
            threshold: mazeThreshold
          });
          endSaveButtonAfter(btn, 'Saved!', 'Save Maze Settings', 1500, () => {
            options.checkMazeConfigChanged();
          });
          setAdminMessage(msg, 'Maze settings saved', 'success');
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          console.error('Failed to save maze config:', e);
          endSaveButton(btn, 'Save Maze Settings');
          options.checkMazeConfigChanged();
        }
      };
    }

    const saveRobotsButton = getById('save-robots-config');
    if (saveRobotsButton) {
      saveRobotsButton.onclick = async function saveRobotsConfig() {
        const msg = getById('admin-msg');
        const btn = this;

        const robotsEnabled = getById('robots-enabled-toggle').checked;
        const crawlDelay = options.readIntegerFieldValue('robots-crawl-delay', msg);
        if (crawlDelay === null) return;

        beginSaveButton(btn);

        try {
          await saveConfigPatch(msg, {
            robots_enabled: robotsEnabled,
            robots_crawl_delay: crawlDelay
          });

          btn.textContent = 'Updated!';
          options.setRobotsSavedState({
            enabled: robotsEnabled,
            crawlDelay: crawlDelay
          });
          const preview = getById('robots-preview');
          if (preview && !preview.classList.contains('hidden')) {
            await options.refreshRobotsPreview();
          }
          endSaveButtonAfter(btn, 'Updated!', 'Save robots serving', 1500, () => {
            options.checkRobotsConfigChanged();
          });
        } catch (e) {
          endSaveButton(btn, 'Error');
          console.error('Failed to save robots config:', e);
          endSaveButtonAfter(btn, undefined, 'Save robots serving', 2000, () => {
            options.checkRobotsConfigChanged();
          });
        }
      };
    }

    const saveAiPolicyButton = getById('save-ai-policy-config');
    if (saveAiPolicyButton) {
      saveAiPolicyButton.onclick = async function saveAiPolicyConfig() {
        const msg = getById('admin-msg');
        const btn = this;

        const blockTraining = getById('robots-block-training-toggle').checked;
        const blockSearch = getById('robots-block-search-toggle').checked;
        const allowSearchEngines = !getById('robots-allow-search-toggle').checked;

        beginSaveButton(btn);

        try {
          await saveConfigPatch(msg, {
            ai_policy_block_training: blockTraining,
            ai_policy_block_search: blockSearch,
            ai_policy_allow_search_engines: allowSearchEngines
          });

          btn.textContent = 'Saved!';
          options.setAiPolicySavedState({
            blockTraining: blockTraining,
            blockSearch: blockSearch,
            allowSearch: getById('robots-allow-search-toggle').checked
          });
          const preview = getById('robots-preview');
          if (preview && !preview.classList.contains('hidden')) {
            await options.refreshRobotsPreview();
          }
          endSaveButtonAfter(btn, 'Saved!', 'Save AI bot policy', 1500, () => {
            options.checkAiPolicyConfigChanged();
          });
        } catch (e) {
          endSaveButton(btn, 'Error');
          console.error('Failed to save AI bot policy:', e);
          endSaveButtonAfter(btn, undefined, 'Save AI bot policy', 2000, () => {
            options.checkAiPolicyConfigChanged();
          });
        }
      };
    }

    const saveGeoScoringButton = getById('save-geo-scoring-config');
    if (saveGeoScoringButton) {
      saveGeoScoringButton.onclick = async function saveGeoScoringConfig() {
        const msg = getById('admin-msg');
        const btn = this;
        const geoState = options.getGeoSavedState();

        if (!geoState.mutable) {
          setAdminMessage(
            msg,
            'GEO settings are read-only while SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false.',
            'warning'
          );
          btn.disabled = true;
          return;
        }

        let geoRisk;
        try {
          geoRisk = parseCountryCodesStrict(getById('geo-risk-list').value);
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          return;
        }

        beginSaveButton(btn);
        try {
          const data = await saveConfigPatch(msg, { geo_risk: geoRisk });
          if (data && data.config) {
            options.updateGeoConfig(data.config);
          } else {
            options.setGeoSavedState({
              ...options.getGeoSavedState(),
              risk: geoRisk.join(','),
              mutable: true
            });
          }
          setAdminMessage(msg, 'GEO scoring saved', 'success');
          endSaveButton(btn, 'Save GEO Scoring');
          options.checkGeoConfigChanged();
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save GEO Scoring');
          options.checkGeoConfigChanged();
        }
      };
    }

    const saveGeoRoutingButton = getById('save-geo-routing-config');
    if (saveGeoRoutingButton) {
      saveGeoRoutingButton.onclick = async function saveGeoRoutingConfig() {
        const msg = getById('admin-msg');
        const btn = this;
        const geoState = options.getGeoSavedState();

        if (!geoState.mutable) {
          setAdminMessage(
            msg,
            'GEO settings are read-only while SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false.',
            'warning'
          );
          btn.disabled = true;
          return;
        }

        let geoAllow;
        let geoChallenge;
        let geoMaze;
        let geoBlock;
        try {
          geoAllow = parseCountryCodesStrict(getById('geo-allow-list').value);
          geoChallenge = parseCountryCodesStrict(getById('geo-challenge-list').value);
          geoMaze = parseCountryCodesStrict(getById('geo-maze-list').value);
          geoBlock = parseCountryCodesStrict(getById('geo-block-list').value);
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          return;
        }

        beginSaveButton(btn);
        try {
          const data = await saveConfigPatch(msg, {
            geo_allow: geoAllow,
            geo_challenge: geoChallenge,
            geo_maze: geoMaze,
            geo_block: geoBlock
          });
          if (data && data.config) {
            options.updateGeoConfig(data.config);
          } else {
            options.setGeoSavedState({
              ...options.getGeoSavedState(),
              allow: geoAllow.join(','),
              challenge: geoChallenge.join(','),
              maze: geoMaze.join(','),
              block: geoBlock.join(','),
              mutable: true
            });
          }
          setAdminMessage(msg, 'GEO routing saved', 'success');
          endSaveButton(btn, 'Save GEO Routing');
          options.checkGeoConfigChanged();
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save GEO Routing');
          options.checkGeoConfigChanged();
        }
      };
    }

    const saveHoneypotButton = getById('save-honeypot-config');
    if (saveHoneypotButton) {
      saveHoneypotButton.onclick = async function saveHoneypotConfig() {
        const msg = getById('admin-msg');
        const btn = this;
        const enabledToggle = getById('honeypot-enabled-toggle');
        const honeypotEnabled = enabledToggle ? enabledToggle.checked : true;
        const field = getById('honeypot-paths');
        let honeypots;

        try {
          honeypots = parseHoneypotPaths(field ? field.value : '');
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          return;
        }

        beginSaveButton(btn);
        try {
          const data = await saveConfigPatch(msg, {
            honeypot_enabled: honeypotEnabled,
            honeypots
          });
          if (data && data.config && typeof options.updateHoneypotConfig === 'function') {
            options.updateHoneypotConfig(data.config);
          } else if (typeof options.setHoneypotSavedState === 'function') {
            options.setHoneypotSavedState({
              enabled: honeypotEnabled,
              values: normalizeList(field ? field.value : '')
            });
            if (typeof options.checkHoneypotConfigChanged === 'function') {
              options.checkHoneypotConfigChanged();
            }
          }
          setAdminMessage(msg, 'Honeypot paths saved', 'success');
          endSaveButton(btn, 'Save Honeypots');
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save Honeypots');
          if (typeof options.checkHoneypotConfigChanged === 'function') {
            options.checkHoneypotConfigChanged();
          }
        }
      };
    }

    const saveBrowserPolicyButton = getById('save-browser-policy-config');
    if (saveBrowserPolicyButton) {
      saveBrowserPolicyButton.onclick = async function saveBrowserPolicyConfig() {
        const msg = getById('admin-msg');
        const btn = this;
        const blockField = getById('browser-block-rules');
        const whitelistField = getById('browser-whitelist-rules');
        let browserBlock;
        let browserWhitelist;

        try {
          browserBlock = parseBrowserRules(blockField ? blockField.value : '');
          browserWhitelist = parseBrowserRules(whitelistField ? whitelistField.value : '');
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          return;
        }

        beginSaveButton(btn);
        try {
          const data = await saveConfigPatch(msg, {
            browser_block: browserBlock,
            browser_whitelist: browserWhitelist
          });
          if (data && data.config && typeof options.updateBrowserPolicyConfig === 'function') {
            options.updateBrowserPolicyConfig(data.config);
          } else if (typeof options.setBrowserPolicySavedState === 'function') {
            options.setBrowserPolicySavedState({
              block: normalizeBrowserRules(blockField ? blockField.value : ''),
              whitelist: normalizeBrowserRules(whitelistField ? whitelistField.value : '')
            });
            if (typeof options.checkBrowserPolicyConfigChanged === 'function') {
              options.checkBrowserPolicyConfigChanged();
            }
          }
          setAdminMessage(msg, 'Browser policy saved', 'success');
          endSaveButton(btn, 'Save Browser Policy');
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save Browser Policy');
          if (typeof options.checkBrowserPolicyConfigChanged === 'function') {
            options.checkBrowserPolicyConfigChanged();
          }
        }
      };
    }

    const saveWhitelistButton = getById('save-whitelist-config');
    if (saveWhitelistButton) {
      saveWhitelistButton.onclick = async function saveWhitelistConfig() {
        const msg = getById('admin-msg');
        const btn = this;
        const networkField = getById('network-whitelist');
        const pathField = getById('path-whitelist');
        const whitelist = parseList(networkField ? networkField.value : '');
        const pathWhitelist = parseList(pathField ? pathField.value : '');

        beginSaveButton(btn);
        try {
          const data = await saveConfigPatch(msg, {
            whitelist,
            path_whitelist: pathWhitelist
          });
          if (data && data.config && typeof options.updateBypassAllowlistConfig === 'function') {
            options.updateBypassAllowlistConfig(data.config);
          } else if (typeof options.setBypassAllowlistSavedState === 'function') {
            options.setBypassAllowlistSavedState({
              network: normalizeList(networkField ? networkField.value : ''),
              path: normalizeList(pathField ? pathField.value : '')
            });
            if (typeof options.checkBypassAllowlistsConfigChanged === 'function') {
              options.checkBypassAllowlistsConfigChanged();
            }
          }
          setAdminMessage(msg, 'Bypass allowlists saved', 'success');
          endSaveButton(btn, 'Save Allowlists');
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save Allowlists');
          if (typeof options.checkBypassAllowlistsConfigChanged === 'function') {
            options.checkBypassAllowlistsConfigChanged();
          }
        }
      };
    }

    const savePowButton = getById('save-pow-config');
    if (savePowButton) {
      savePowButton.onclick = async function savePowConfig() {
        const btn = this;
        const msg = getById('admin-msg');

        const powEnabled = getById('pow-enabled-toggle').checked;
        const powDifficulty = options.readIntegerFieldValue('pow-difficulty', msg);
        const powTtl = options.readIntegerFieldValue('pow-ttl', msg);
        if (powDifficulty === null || powTtl === null) return;

        beginSaveButton(btn);

        try {
          await saveConfigPatch(msg, {
            pow_enabled: powEnabled,
            pow_difficulty: powDifficulty,
            pow_ttl_seconds: powTtl
          });

          options.setPowSavedState({
            enabled: powEnabled,
            difficulty: powDifficulty,
            ttl: powTtl,
            mutable: true
          });
          setAdminMessage(msg, 'PoW settings saved', 'success');
          endSaveButton(btn, 'Save PoW Settings');
          options.checkPowConfigChanged();
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save PoW Settings');
          options.checkPowConfigChanged();
        }
      };
    }

    const saveChallengePuzzleButton = getById('save-challenge-puzzle-config');
    if (saveChallengePuzzleButton) {
      saveChallengePuzzleButton.onclick = async function saveChallengePuzzleConfig() {
        const btn = this;
        const msg = getById('admin-msg');
        const challengeEnabled = getById('challenge-puzzle-enabled-toggle').checked;
        const transformCount = options.readIntegerFieldValue('challenge-puzzle-transform-count', msg);
        if (transformCount === null) return;

        beginSaveButton(btn);
        try {
          await saveConfigPatch(msg, {
            challenge_puzzle_enabled: challengeEnabled,
            challenge_puzzle_transform_count: transformCount
          });
          if (typeof options.setChallengePuzzleSavedState === 'function') {
            options.setChallengePuzzleSavedState({
              enabled: challengeEnabled,
              count: transformCount,
              mutable: true
            });
          }
          setAdminMessage(msg, 'Challenge puzzle settings saved', 'success');
          endSaveButton(btn, 'Save Challenge Puzzle');
          if (typeof options.checkChallengePuzzleConfigChanged === 'function') {
            options.checkChallengePuzzleConfigChanged();
          }
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save Challenge Puzzle');
          if (typeof options.checkChallengePuzzleConfigChanged === 'function') {
            options.checkChallengePuzzleConfigChanged();
          }
        }
      };
    }

    const saveBotnessButton = getById('save-botness-config');
    if (saveBotnessButton) {
      saveBotnessButton.onclick = async function saveBotnessConfig() {
        const btn = this;
        const msg = getById('admin-msg');

        const challengeThreshold = options.readIntegerFieldValue('challenge-puzzle-threshold', msg);
        const mazeThreshold = options.readIntegerFieldValue('maze-threshold-score', msg);
        const weightJsRequired = options.readIntegerFieldValue('weight-js-required', msg);
        const weightGeoRisk = options.readIntegerFieldValue('weight-geo-risk', msg);
        const weightRateMedium = options.readIntegerFieldValue('weight-rate-medium', msg);
        const weightRateHigh = options.readIntegerFieldValue('weight-rate-high', msg);

        if (
          challengeThreshold === null ||
          mazeThreshold === null ||
          weightJsRequired === null ||
          weightGeoRisk === null ||
          weightRateMedium === null ||
          weightRateHigh === null
        ) {
          return;
        }

        beginSaveButton(btn);

        try {
          await saveConfigPatch(msg, {
            challenge_puzzle_risk_threshold: challengeThreshold,
            botness_maze_threshold: mazeThreshold,
            botness_weights: {
              js_required: weightJsRequired,
              geo_risk: weightGeoRisk,
              rate_medium: weightRateMedium,
              rate_high: weightRateHigh
            }
          });

          options.setBotnessSavedState({
            challengeThreshold: challengeThreshold,
            mazeThreshold: mazeThreshold,
            weightJsRequired: weightJsRequired,
            weightGeoRisk: weightGeoRisk,
            weightRateMedium: weightRateMedium,
            weightRateHigh: weightRateHigh,
            mutable: true
          });
          setAdminMessage(msg, 'Botness scoring saved', 'success');
          endSaveButton(btn, 'Save Botness Settings');
          options.checkBotnessConfigChanged();
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save Botness Settings');
          options.checkBotnessConfigChanged();
        }
      };
    }

    const saveCdpButton = getById('save-cdp-config');
    if (saveCdpButton) {
      saveCdpButton.onclick = async function saveCdpConfig() {
        const msg = getById('admin-msg');
        const btn = this;

        const cdpEnabled = getById('cdp-enabled-toggle').checked;
        const cdpAutoBan = getById('cdp-auto-ban-toggle').checked;
        const cdpThreshold = parseFloat(getById('cdp-threshold-slider').value);

        beginSaveButton(btn);

        try {
          await saveConfigPatch(msg, {
            cdp_detection_enabled: cdpEnabled,
            cdp_auto_ban: cdpAutoBan,
            cdp_detection_threshold: cdpThreshold
          });

          btn.textContent = 'Saved!';
          options.setCdpSavedState({
            enabled: cdpEnabled,
            autoBan: cdpAutoBan,
            threshold: cdpThreshold
          });
          endSaveButtonAfter(btn, 'Saved!', 'Save CDP Settings', 1500, () => {
            options.checkCdpConfigChanged();
          });
        } catch (e) {
          endSaveButton(btn, 'Error');
          console.error('Failed to save CDP config:', e);
          endSaveButtonAfter(btn, undefined, 'Save CDP Settings', 2000, () => {
            options.checkCdpConfigChanged();
          });
        }
      };
    }

    const saveEdgeModeButton = getById('save-edge-integration-mode-config');
    if (saveEdgeModeButton) {
      saveEdgeModeButton.onclick = async function saveEdgeIntegrationModeConfig() {
        const btn = this;
        const msg = getById('admin-msg');
        const modeSelect = getById('edge-integration-mode-select');
        const mode = String(modeSelect ? modeSelect.value : '').trim().toLowerCase();

        beginSaveButton(btn);
        try {
          const data = await saveConfigPatch(msg, { edge_integration_mode: mode });
          if (data && data.config && typeof options.updateEdgeIntegrationModeConfig === 'function') {
            options.updateEdgeIntegrationModeConfig(data.config);
          } else {
            options.setEdgeIntegrationModeSavedState({ mode });
            options.checkEdgeIntegrationModeChanged();
          }

          setAdminMessage(msg, 'Edge integration mode saved', 'success');
          endSaveButton(btn, 'Save Edge Integration Mode');
          options.checkEdgeIntegrationModeChanged();
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save Edge Integration Mode');
          options.checkEdgeIntegrationModeChanged();
        }
      };
    }

    const saveRateLimitButton = getById('save-rate-limit-config');
    if (saveRateLimitButton) {
      saveRateLimitButton.onclick = async function saveRateLimitConfig() {
        const btn = this;
        const msg = getById('admin-msg');
        const rateLimit = options.readIntegerFieldValue('rate-limit-threshold', msg);
        if (rateLimit === null) return;

        beginSaveButton(btn);
        try {
          await saveConfigPatch(msg, { rate_limit: rateLimit });
          options.setRateLimitSavedState({ value: rateLimit });
          applyStatusPatch({ rateLimit });
          setAdminMessage(msg, 'Rate limit saved', 'success');
          endSaveButton(btn, 'Save Rate Limit');
          options.checkRateLimitConfigChanged();
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save Rate Limit');
          options.checkRateLimitConfigChanged();
        }
      };
    }

    const saveJsRequiredButton = getById('save-js-required-config');
    if (saveJsRequiredButton) {
      saveJsRequiredButton.onclick = async function saveJsRequiredConfig() {
        const btn = this;
        const msg = getById('admin-msg');
        const enforced = getById('js-required-enforced-toggle').checked;

        beginSaveButton(btn);
        try {
          await saveConfigPatch(msg, { js_required_enforced: enforced });
          options.setJsRequiredSavedState({ enforced });
          applyStatusPatch({ jsRequiredEnforced: enforced });
          setAdminMessage(msg, 'JS Required setting saved', 'success');
          endSaveButton(btn, 'Save JS Required');
          options.checkJsRequiredConfigChanged();
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save JS Required');
          options.checkJsRequiredConfigChanged();
        }
      };
    }

    const saveDurationsButton = getById('save-durations-btn');
    if (saveDurationsButton) {
      saveDurationsButton.onclick = async function saveDurations() {
        const msg = getById('admin-msg');
        const btn = this;

        const banDurations = {
          honeypot: options.readBanDurationSeconds('honeypot'),
          rate_limit: options.readBanDurationSeconds('rateLimit'),
          browser: options.readBanDurationSeconds('browser'),
          cdp: options.readBanDurationSeconds('cdp'),
          admin: options.readBanDurationSeconds('admin')
        };

        if (
          banDurations.honeypot === null ||
          banDurations.rate_limit === null ||
          banDurations.browser === null ||
          banDurations.cdp === null ||
          banDurations.admin === null
        ) {
          return;
        }

        setAdminMessage(msg, 'Saving ban durations...', 'info');
        beginSaveButton(btn);

        try {
          const data = await saveConfigPatch(msg, { ban_durations: banDurations });
          const saved = data && data.config && data.config.ban_durations
            ? data.config.ban_durations
            : banDurations;
          options.updateBanDurations({ ban_durations: saved });
          setAdminMessage(msg, 'Ban durations saved', 'success');
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save Durations');
          options.checkBanDurationsChanged();
        }
      };
    }

    const saveAdvancedConfigButton = getById('save-advanced-config');
    if (saveAdvancedConfigButton) {
      saveAdvancedConfigButton.onclick = async function saveAdvancedConfig() {
        const msg = getById('admin-msg');
        const btn = this;
        const patch = typeof options.readAdvancedConfigPatch === 'function'
          ? options.readAdvancedConfigPatch(msg)
          : null;
        if (!patch) return;

        beginSaveButton(btn);

        try {
          const data = await saveConfigPatch(msg, patch);
          if (data && data.config && typeof options.setAdvancedConfigFromConfig === 'function') {
            options.setAdvancedConfigFromConfig(data.config, false);
          } else if (typeof options.checkAdvancedConfigChanged === 'function') {
            options.checkAdvancedConfigChanged();
          }
          setAdminMessage(msg, 'Advanced config patch saved', 'success');
          endSaveButton(btn, 'Save Advanced Config');
          if (typeof options.checkAdvancedConfigChanged === 'function') {
            options.checkAdvancedConfigChanged();
          }
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          endSaveButton(btn, 'Save Advanced Config');
          if (typeof options.checkAdvancedConfigChanged === 'function') {
            options.checkAdvancedConfigChanged();
          }
        }
      };
    }

    const testModeToggle = getById('test-mode-toggle');
    if (testModeToggle) {
      testModeToggle.addEventListener('change', async function onTestModeChange() {
        const msg = getById('admin-msg');
        if (!options.getAdminContext(msg)) {
          this.checked = !this.checked;
          return;
        }
        const testMode = this.checked;

        setAdminMessage(msg, `${testMode ? 'Enabling' : 'Disabling'} test mode...`, 'info');

        try {
          const data = await saveConfigPatch(msg, { test_mode: testMode });
          setAdminMessage(msg, `Test mode ${data.config.test_mode ? 'enabled' : 'disabled'}`, 'success');
          timerSetTimeout(() => options.refreshDashboard(), 500);
        } catch (e) {
          setAdminMessage(msg, `Error: ${e.message}`, 'error');
          this.checked = !testMode;
        }
      });
    }
}

export {
  bind,
  flattenBindOptions as _flattenBindOptions,
  normalizeContextOptions as _normalizeContextOptions
};
