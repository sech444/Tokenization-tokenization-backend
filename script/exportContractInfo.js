import fs from "fs";
import path from "path";
import { execSync } from "child_process";

const deploymentsFile = path.join("deployments", "31337.json");
const outputFile = path.join("deployments", "contracts-info.json");

// Load deployed contract addresses
const deployments = JSON.parse(fs.readFileSync(deploymentsFile, "utf-8"));

const result = {};

for (const [name, address] of Object.entries(deployments)) {
  try {
    // ✅ simpler: just use contract name, Forge finds the file
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
        const inputs = item.inputs.map((i) => i.type).join(",");
        functions.push(`${item.name}(${inputs})`);
        if (item.stateMutability === "view" || item.stateMutability === "pure") {
          variables.push(item.name);
        }
      } else if (item.type === "event") {
        const inputs = item.inputs.map((i) => i.type).join(",");
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

// Save to JSON file
fs.writeFileSync(outputFile, JSON.stringify(result, null, 2));
console.log(`✅ Exported contract info to ${outputFile}`);
