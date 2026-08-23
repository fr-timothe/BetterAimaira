import { cpSync, existsSync } from "node:fs";
import { resolve } from "node:path";

const sourceDirectory = resolve("src-tauri", "icons", "android");
const targetDirectory = resolve("src-tauri", "gen", "android", "app", "src", "main", "res");

if (!existsSync(sourceDirectory)) {
	throw new Error(`Android icon source directory is missing: ${sourceDirectory}`);
}

if (!existsSync(targetDirectory)) {
	throw new Error(`Generated Android resource directory is missing: ${targetDirectory}`);
}

for (const directory of [
	"mipmap-hdpi",
	"mipmap-mdpi",
	"mipmap-xhdpi",
	"mipmap-xxhdpi",
	"mipmap-xxxhdpi"
]) {
	cpSync(resolve(sourceDirectory, directory), resolve(targetDirectory, directory), {
		recursive: true,
		force: true
	});
}
