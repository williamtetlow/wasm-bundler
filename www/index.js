import * as wasm from "wasm-bundler";

const result = wasm.bundle();

document.getElementById("result").textContent = result;
