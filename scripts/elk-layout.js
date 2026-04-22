#!/usr/bin/env node
// ELK layout engine wrapper for lox config layout
// Usage: node scripts/elk-layout.js < graph.json > layout.json
//
// Input: ELK JSON graph format with nodes, ports, edges
// Output: Same graph with x/y coordinates assigned

const ELK = require('elkjs');

async function main() {
  let input = '';
  for await (const chunk of process.stdin) {
    input += chunk;
  }

  const graph = JSON.parse(input);

  const elk = new ELK();
  const layout = await elk.layout(graph);

  // Flatten: extract node positions
  const positions = {};
  function extractPositions(node, offsetX = 0, offsetY = 0) {
    if (node.id && node.x !== undefined) {
      positions[node.id] = {
        x: Math.round(node.x + offsetX),
        y: Math.round(node.y + offsetY),
        width: Math.round(node.width || 0),
        height: Math.round(node.height || 0),
      };
    }
    if (node.children) {
      for (const child of node.children) {
        extractPositions(child, (node.x || 0) + offsetX, (node.y || 0) + offsetY);
      }
    }
  }
  extractPositions(layout);

  console.log(JSON.stringify(positions, null, 2));
}

main().catch(e => {
  console.error(e.message);
  process.exit(1);
});
