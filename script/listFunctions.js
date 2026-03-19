// // listFunctions("FeeManager");
// // run "node script/listFunctions.js FeeManager" to get a list of functions


// import fs from "fs";
// import path from "path";

// const DEPLOYMENTS_FILE = "deployments/31337.json";

// function loadDeployments() {
//   if (!fs.existsSync(DEPLOYMENTS_FILE)) {
//     console.error(`❌ Deployments file not found: ${DEPLOYMENTS_FILE}`);
//     process.exit(1);
//   }
//   return JSON.parse(fs.readFileSync(DEPLOYMENTS_FILE, "utf8"));
// }

// function getAbi(contractName) {
//   const artifactPath = path.resolve(`out/${contractName}.sol/${contractName}.json`);
//   if (!fs.existsSync(artifactPath)) {
//     console.error(`❌ ABI not found for ${contractName}`);
//     process.exit(1);
//   }
//   const artifact = JSON.parse(fs.readFileSync(artifactPath, "utf8"));
//   return artifact.abi;
// }

// function listFunctions(contractName, address) {
//   const abi = getAbi(contractName);
//   const functions = abi.filter(x => x.type === "function").map(f => f.name);

//   console.log(`\n📌 ${contractName}`);
//   console.log(`   Address: ${address}`);
//   console.log("   Functions:");
//   functions.forEach(fn => console.log("    - " + fn));
// }

// // main
// const deployments = loadDeployments();
// const contractName = process.argv[2];

// if (!contractName) {
//   console.log("Usage: node script/listFunctions.js <ContractName>");
//   console.log("Available contracts:", Object.keys(deployments).join(", "));
//   process.exit(1);
// }

// const address = deployments[contractName];
// if (!address) {
//   console.error(`❌ No address found for contract: ${contractName}`);
//   process.exit(1);
// }

// listFunctions(contractName, address);


// import fs from "fs";
// import path from "path";

// const DEPLOYMENTS_FILE = "deployments/31337.json";
// const OUTPUT_FILE = "deployments/functions-31337.txt";

// function loadDeployments() {
//   if (!fs.existsSync(DEPLOYMENTS_FILE)) {
//     console.error(`❌ Deployments file not found: ${DEPLOYMENTS_FILE}`);
//     process.exit(1);
//   }
//   return JSON.parse(fs.readFileSync(DEPLOYMENTS_FILE, "utf8"));
// }

// function getAbi(contractName) {
//   const artifactPath = path.resolve(`out/${contractName}.sol/${contractName}.json`);
//   if (!fs.existsSync(artifactPath)) {
//     console.error(`❌ ABI not found for ${contractName}`);
//     return null;
//   }
//   const artifact = JSON.parse(fs.readFileSync(artifactPath, "utf8"));
//   return artifact.abi;
// }

// function listFunctions(contractName, address) {
//   const abi = getAbi(contractName);
//   if (!abi) return "";

//   const functions = abi
//     .filter(x => x.type === "function")
//     .map(f => `    - ${f.name}`);

//   let result = `📌 ${contractName}\n   Address: ${address}\n   Functions:\n${functions.join("\n")}\n`;
//   console.log(result);
//   return result;
// }

// // main
// const deployments = loadDeployments();
// const contractName = process.argv[2];

// let output = "";

// if (!contractName || contractName === "all") {
//   console.log("Listing all contracts...\n");
//   Object.entries(deployments).forEach(([name, addr]) => {
//     output += listFunctions(name, addr) + "\n";
//   });
// } else {
//   const address = deployments[contractName];
//   if (!address) {
//     console.error(`❌ No address found for contract: ${contractName}`);
//     process.exit(1);
//   }
//   output += listFunctions(contractName, address) + "\n";
// }

// // Save to file
// fs.writeFileSync(OUTPUT_FILE, output, "utf8");
// console.log(`\n✅ Output saved to ${OUTPUT_FILE}`);


import fs from "fs";
import path from "path";

const DEPLOYMENTS_FILE = "deployments/31337.json";
const OUTPUT_FILE = "deployments/functions-31337.txt";

function loadDeployments() {
  if (!fs.existsSync(DEPLOYMENTS_FILE)) {
    console.error(`❌ Deployments file not found: ${DEPLOYMENTS_FILE}`);
    process.exit(1);
  }
  return JSON.parse(fs.readFileSync(DEPLOYMENTS_FILE, "utf8"));
}

function getAbi(contractName) {
  const artifactPath = path.resolve(`out/${contractName}.sol/${contractName}.json`);
  if (!fs.existsSync(artifactPath)) {
    console.error(`❌ ABI not found for ${contractName}`);
    return null;
  }
  const artifact = JSON.parse(fs.readFileSync(artifactPath, "utf8"));
  return artifact.abi;
}

function listInterface(contractName, address) {
  const abi = getAbi(contractName);
  if (!abi) return "";

  const functions = abi
    .filter(x => x.type === "function")
    .map(f => `    - ${f.name}(${f.inputs.map(i => i.type).join(", ")})`);

  const events = abi
    .filter(x => x.type === "event")
    .map(e => `    - ${e.name}(${e.inputs.map(i => i.type).join(", ")})`);

  const variables = abi
    .filter(x => x.type === "function" && x.stateMutability === "view" && x.inputs.length === 0)
    .map(v => `    - ${v.name} (view)`);

  let result = `📌 ${contractName}\n   Address: ${address}\n   Functions:\n${functions.join("\n") || "    - None"}\n   Events:\n${events.join("\n") || "    - None"}\n   Public Variables:\n${variables.join("\n") || "    - None"}\n`;
  
  console.log(result);
  return result;
}

// main
const deployments = loadDeployments();
const contractName = process.argv[2];

let output = "";

if (!contractName || contractName === "all") {
  console.log("Listing all contracts...\n");
  Object.entries(deployments).forEach(([name, addr]) => {
    output += listInterface(name, addr) + "\n";
  });
} else {
  const address = deployments[contractName];
  if (!address) {
    console.error(`❌ No address found for contract: ${contractName}`);
    process.exit(1);
  }
  output += listInterface(contractName, address) + "\n";
}

// Save to file
fs.writeFileSync(OUTPUT_FILE, output, "utf8");
console.log(`\n✅ Output saved to ${OUTPUT_FILE}`);

