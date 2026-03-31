#!/usr/bin/env node

import assert from "node:assert/strict";

import {
  applyChallengePuzzleWrongOutput,
  classifyMazeDocument,
  mergeAgenticSessionPaths,
  summarizeBrowserSecondaryTraffic,
  validateAllowBrowserAllowlistResponse,
} from "./adversarial_browser_driver.mjs";

class FakeLocator {
  constructor({ visible = false, count = 0, inputValue = "", onEvaluate = null } = {}) {
    this.visible = visible;
    this.countValue = count;
    this.inputValueValue = inputValue;
    this.onEvaluate = onEvaluate;
  }

  async isVisible() {
    return this.visible;
  }

  async count() {
    return this.countValue;
  }

  async inputValue() {
    return this.inputValueValue;
  }

  async evaluate(callback, value) {
    if (this.onEvaluate) {
      this.onEvaluate(value);
    }
    return callback({ value: this.inputValueValue }, value);
  }
}

class FakePage {
  constructor(locators) {
    this.locators = locators;
  }

  locator(selector) {
    const locator = this.locators.get(selector);
    if (!locator) {
      throw new Error(`unexpected selector:${selector}`);
    }
    return locator;
  }
}

async function testHiddenPuzzleOutputFieldIsAccepted() {
  const writes = [];
  const evidence = { challenge_dom_path: [] };
  const page = new FakePage(
    new Map([
      ["#challenge-output-grid", new FakeLocator({ visible: true, count: 1 })],
      [
        "#challenge-output",
        new FakeLocator({
          visible: false,
          count: 1,
          inputValue: "0000",
          onEvaluate: (value) => writes.push(value),
        }),
      ],
    ]),
  );

  const wrongOutput = await applyChallengePuzzleWrongOutput(page, evidence);

  assert.equal(wrongOutput, "1000");
  assert.deepEqual(writes, ["1000"]);
  assert.deepEqual(evidence.challenge_dom_path, [
    "read:#challenge-output-grid",
    "write:#challenge-output",
  ]);
}

async function testMissingPuzzleOutputFieldFails() {
  const page = new FakePage(
    new Map([
      ["#challenge-output-grid", new FakeLocator({ visible: true, count: 1 })],
      ["#challenge-output", new FakeLocator({ visible: false, count: 0 })],
    ]),
  );

  await assert.rejects(
    () => applyChallengePuzzleWrongOutput(page, { challenge_dom_path: [] }),
    /browser_puzzle_output_field_missing/,
  );
}

await testHiddenPuzzleOutputFieldIsAccepted();
await testMissingPuzzleOutputFieldFails();

function testAllowBrowserAllowlistRejectsVerificationInterstitial() {
  assert.throws(
    () =>
      validateAllowBrowserAllowlistResponse(
        200,
        "<html><body>Verifying...<script>/* challenge */</script></body></html>",
      ),
    /browser_allow_expected_clean_allow/,
  );
}

function testAllowBrowserAllowlistAcceptsGatewayFailClosedFallback() {
  const result = validateAllowBrowserAllowlistResponse(
    502,
    "<html><body>Gateway forwarding unavailable</body></html>",
  );
  assert.deepEqual(result, {
    observed_outcome: "allow",
    detail: "gateway_forwarding_unavailable",
  });
}

function testAllowBrowserAllowlistAcceptsCleanAllowBody() {
  const result = validateAllowBrowserAllowlistResponse(
    200,
    "<html><body><h1>Welcome</h1></body></html>",
  );
  assert.deepEqual(result, {
    observed_outcome: "allow",
    detail: "ok",
  });
}

testAllowBrowserAllowlistRejectsVerificationInterstitial();
testAllowBrowserAllowlistAcceptsGatewayFailClosedFallback();
testAllowBrowserAllowlistAcceptsCleanAllowBody();

function testClassifyMazeDocumentRecognizesMazePages() {
  assert.equal(
    classifyMazeDocument(200, '<html><body><a data-link-kind="maze" href="/_/abc/next">next</a></body></html>'),
    "maze",
  );
}

function testClassifyMazeDocumentRecognizesChallengeFallback() {
  assert.equal(
    classifyMazeDocument(200, "<html><body>Verifying<script>document.cookie = 'js_verified=1'</script></body></html>"),
    "challenge",
  );
}

function testClassifyMazeDocumentRecognizesBlockFallback() {
  assert.equal(
    classifyMazeDocument(403, "<html><body><h1>Access Blocked</h1></body></html>"),
    "block",
  );
}

testClassifyMazeDocumentRecognizesMazePages();
testClassifyMazeDocumentRecognizesChallengeFallback();
testClassifyMazeDocumentRecognizesBlockFallback();

function testMergeAgenticSessionPathsRootsAndDeduplicatesWithinBudget() {
  const merged = mergeAgenticSessionPaths(
    ["/research/", "/research/"],
    ["/plans/", "/work/"],
    3,
  );
  assert.deepEqual(merged, ["/", "/research/", "/plans/"]);
}

testMergeAgenticSessionPathsRootsAndDeduplicatesWithinBudget();

function testSummarizeBrowserSecondaryTrafficSeparatesBackgroundAndSubresources() {
  const summary = summarizeBrowserSecondaryTraffic([
    {
      method: "GET",
      path: "/",
      request_kind: "top_level",
      resource_type: "document",
    },
    {
      method: "GET",
      path: "/static/site.css",
      request_kind: "subresource",
      resource_type: "stylesheet",
    },
    {
      method: "GET",
      path: "/static/app.js",
      request_kind: "subresource",
      resource_type: "script",
    },
    {
      method: "POST",
      path: "/browser-beacon",
      request_kind: "background",
      resource_type: "fetch",
    },
  ]);

  assert.deepEqual(summary, {
    secondary_capture_mode: "same_origin_request_events",
    secondary_request_count: 3,
    background_request_count: 1,
    subresource_request_count: 2,
  });
}

testSummarizeBrowserSecondaryTrafficSeparatesBackgroundAndSubresources();
