export const categories = {
  Abstract: ["◈", "✺", "✿", "◎", "◒"],
  Animals: ["🦊", "🐼", "🐯", "🐸", "🦉"],
  People: ["🧑‍🚀", "🧑‍🎨", "🧙", "🥷", "🧑‍💻"],
  Space: ["🪐", "🚀", "🌙", "☄️", "🛸"],
};
const colors = ["#314e52", "#493d63", "#654336", "#345341", "#3a456c"];

export function preset(symbol: string, index: number): string {
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 256;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Image editing is unavailable in this browser");
  context.fillStyle = colors[index % colors.length];
  context.fillRect(0, 0, 256, 256);
  context.fillStyle = "#f1e5d0";
  context.font = "128px sans-serif";
  context.textAlign = "center";
  context.textBaseline = "middle";
  context.fillText(symbol, 128, 136);
  return canvas.toDataURL("image/png");
}
