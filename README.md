# wasm-bundler

A browser based bundler, using SWC.

![Example showing using calculator and output](https://github.com/williamtetlow/wasm-bundler/blob/main/example.png)

# Setup

- [rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [npm](https://www.npmjs.com/get-npm)

1. Build wasm package

```
wasm-pack build
```

2. Install node modules

```
cd www
npm install
```

# Running

```
npm run build
npm run serve
```

# Dev

1. Open a shell to run webpack

```
cd www
npm run dev
```

2. Make changes to wasm package and build again

```
wasm-pack build
```
