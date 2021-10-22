// const fs = require('fs');
import fs from "fs";

function ls(path) {
  return fs.readdirSync(path).filter(function (file) {
    return fs.statSync(path + "/" + file).isDirectory();
  });
}

console.log(ls(".").join("\n"));
