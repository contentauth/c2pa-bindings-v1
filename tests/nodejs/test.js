const ffi = require('ffi-napi')
const ref = require("ref-napi");
const path = require('path');

console.log(__dirname);
var p = path.join(__dirname,'../../target/release/libc2pa_uniffi.dylib');

var lib = ffi.Library(p, {
    c2pa_version: [ 'char *' , [ ]],
    c2pa_verify_from_file: [ 'char *' , ['string']],
    c2pa_add_manifest_to_file: ['char *', ['string','string','string','string','byte']],
    c2pa_release_string: ['void', ['char *']]
  });

function parseResult(cstr) {
  result = JSON.parse(cstr.readCString());
  lib.c2pa_release_string(cstr)
  if (result.error) {
    throw Error(result.error.message, {cause: result.error.code})
  } 
  return JSON.parse(result.ok)
}

function c2paVersion() {
  cstr = lib.c2pa_version();
  result = cstr.readCString();
  lib.c2pa_release_string(cstr)
  return result
}

function c2paVerifyFile(path) {
  cstr = lib.c2pa_verify_from_file(path);
  return parseResult(cstr)
}

function c2paAddManifest(source, dest, manifest, sign_info, options) {
  cstr = lib.c2pa_add_manifest_to_file(source, dest, manifest, sign_info, options);
  return parseResult(cstr)
}

console.log(c2paVersion());

try {
  store = c2paVerifyFile("../fixtures/C.jpg")
  console.log(store);
  manifest = store["manifests"][store["active_manifest"]]
  console.log(manifest)
  console.log(manifest["assertions"])
} catch (error) {
  console.error(error.cause, error.message);
}
