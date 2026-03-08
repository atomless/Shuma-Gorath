#!/usr/bin/env node

import assert from "node:assert/strict";

import { applyChallengePuzzleWrongOutput } from "./adversarial_browser_driver.mjs";

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
