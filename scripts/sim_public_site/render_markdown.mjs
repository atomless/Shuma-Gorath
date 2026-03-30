import process from "node:process";
import { HtmlRenderer, Parser } from "commonmark";

let input = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => {
  input += chunk;
});
process.stdin.on("end", () => {
  const parser = new Parser();
  const renderer = new HtmlRenderer({ safe: true });
  const parsed = parser.parse(input);
  process.stdout.write(renderer.render(parsed));
});
