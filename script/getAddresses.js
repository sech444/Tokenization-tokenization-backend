// import fs from "fs";

// function getAllAddresses(chainId = "31337") {
//   const file = `broadcast/Deploy.s.sol/${chainId}/run-latest.json`;
//   if (!fs.existsSync(file)) {
//     console.error(`❌ File not found: ${file}`);
//     process.exit(1);
//   }

//   const data = JSON.parse(fs.readFileSync(file));

//   const addresses = {};
//   for (const tx of data.transactions || []) {
//     if (tx.contractName && tx.contractAddress) {
//       addresses[tx.contractName] = tx.contractAddress;
//     }
//   }
//   return addresses;
// }

// function saveAddresses(chainId = "31337") {
//   const addresses = getAllAddresses(chainId);
//   const outPath = `deployments/${chainId}.json`;

//   fs.mkdirSync("deployments", { recursive: true });
//   fs.writeFileSync(outPath, JSON.stringify(addresses, null, 2));

//   console.log(`✅ Addresses saved to ${outPath}`);
// }

// const chainId = process.argv[2] || "31337";
// saveAddresses(chainId);


const fs = require("fs");

function getAllAddresses(chainId = "31337") {
  const file = `broadcast/Deploy.s.sol/${chainId}/run-latest.json`;
  if (!fs.existsSync(file)) {
    console.error(`❌ File not found: ${file}`);
    process.exit(1);
  }

  const data = JSON.parse(fs.readFileSync(file));

  const addresses = {};
  for (const tx of data.transactions || []) {
    if (tx.contractName && tx.contractAddress) {
      addresses[tx.contractName] = tx.contractAddress;
    }
  }
  return addresses;
}

function saveAddresses(chainId = "31337") {
  const addresses = getAllAddresses(chainId);
  const outPath = `deployments/${chainId}.json`;

  fs.mkdirSync("deployments", { recursive: true });
  fs.writeFileSync(outPath, JSON.stringify(addresses, null, 2));

  console.log(`✅ Addresses saved to ${outPath}`);
}

const chainId = process.argv[2] || "31337";
saveAddresses(chainId);

