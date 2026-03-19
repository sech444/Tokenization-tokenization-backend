// generate-info.js
import fs from "fs";
import path from "path";
import { execSync } from "child_process";

// Correct path to your Foundry deployment broadcast file for chain 137
const broadcastFile = path.join("broadcast", "Deploy.s.sol", "137", "run-latest.json");
const outputFile = path.join("deployments", "contracts-info.json");

console.log(`Reading deployment data from: ${broadcastFile}`);

if (!fs.existsSync(broadcastFile)) {
  console.error(`❌ Error: Deployment file not found at ${broadcastFile}`);
  console.error("Please make sure you have run your deployment script successfully on chain 137.");
  process.exit(1);
}

// Load and parse the broadcast JSON file
const broadcastData = JSON.parse(fs.readFileSync(broadcastFile, "utf-8"));
const deployedContracts = {};

// Extract contract names and addresses from the 'transactions' array
broadcastData.transactions.forEach(tx => {
  if (tx.transactionType === "CREATE" && tx.contractName && tx.contractAddress) {
    // Avoid duplicates, keeping the last deployed instance
    deployedContracts[tx.contractName] = tx.contractAddress;
  }
});

console.log("Found deployed contracts:", Object.keys(deployedContracts));

const result = {};

for (const [name, address] of Object.entries(deployedContracts)) {
  try {
    console.log(`Inspecting ${name}...`);
    // Use contract name to get ABI. Forge finds the file path.
    const abiRaw = execSync(
      `forge inspect ${name} abi --json`,
      { encoding: "utf-8" }
    );
    const abi = JSON.parse(abiRaw);

    const functions = [];
    const events = [];
    const variables = [];

    abi.forEach((item) => {
      if (item.type === "function") {
        const inputs = item.inputs.map((i) => `${i.type} ${i.name}`).join(", ");
        const outputs = item.outputs.map((o) => o.type).join(", ");
        const mutability = item.stateMutability !== "nonpayable" ? ` ${item.stateMutability}` : "";
        const returns = outputs ? ` returns (${outputs})` : "";
        functions.push(`${item.name}(${inputs})${mutability}${returns}`);
        
        // Also capture public state variables which are exposed as view functions
        if (item.stateMutability === "view" && item.inputs.length === 0) {
          variables.push(`${item.name}: ${item.outputs[0].type}`);
        }
      } else if (item.type === "event") {
        const inputs = item.inputs.map((i) => `${i.type}${i.indexed ? " indexed" : ""} ${i.name}`).join(", ");
        events.push(`${item.name}(${inputs})`);
      }
    });

    result[name] = {
      address,
      functions,
      events,
      variables,
    };
  } catch (err) {
    console.error(`❌ Error inspecting ${name}: ${err.message}`);
  }
}

// Ensure the output directory exists
const outputDir = path.dirname(outputFile);
if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

// Save to JSON file
fs.writeFileSync(outputFile, JSON.stringify(result, null, 2));
console.log(`\n✅ Exported contract info to ${outputFile}`);