// Rolled names for the dice button in the name prompts. Lists get two words, projects one.
const LIST_FIRST = ["Golden", "Silver", "Quiet", "Gilded", "Misty", "Amber", "Coral", "Velvet", "Cobalt", "Sunny", "Maple", "Hollow"];
const LIST_SECOND = ["Oasis", "Harbor", "Meadow", "Lantern", "Orchard", "Canyon", "Lagoon", "Summit", "Garden", "Fjord"];
const PROJECT = ["Nereid", "Titan", "Halcyon", "Orion", "Solace", "Atlas", "Meridian", "Cinder", "Tundra", "Zephyr", "Lumen", "Juniper"];

function pick<T>(items: T[]): T {
  return items[Math.floor(Math.random() * items.length)];
}

export function rollListName(): string {
  return `${pick(LIST_FIRST)} ${pick(LIST_SECOND)}`;
}

export function rollProjectName(): string {
  return pick(PROJECT);
}
