import * as wasm from "wasm-bundler";

const entryPoint = "main.js";

const initialFiles = {
  [entryPoint]: `import { A, FOO, add } from './a';

console.log(A, FOO);
  
console.log(add(1, 2));`,
  "./a.js": `export const FOO = 1;

export function add(x, y) {
  return x + y;
}
export class A {
    foo() {
  }
}`,
};

// set defaults
document.getElementsByName("filename1")[0].value = "main.js";
document.getElementsByName("content1")[0].value = initialFiles["main.js"];
document.getElementsByName("filename2")[0].value = "./a.js";
document.getElementsByName("content2")[0].value = initialFiles["./a.js"];

// add listeners
document.getElementsByName("content1")[0].addEventListener("input", (e) => {
  const newContents = e.currentTarget.value;

  bundler.update_file("main.js", newContents);
});

document.getElementsByName("content2")[0].addEventListener("input", (e) => {
  const newContents = e.currentTarget.value;

  bundler.update_file("./a.js", newContents);
});

document.getElementById("bundle").addEventListener("click", () => {
  const result = bundler.bundle();

  document.getElementById("result").textContent = result;
});

const bundler = wasm.WasmBundler.new(entryPoint);

for (const [filename, content] of Object.entries(initialFiles)) {
  bundler.add_file(filename, content);
}

const result = bundler.bundle();

document.getElementById("result").textContent = result;

/**
 * We send as a param to
 */
