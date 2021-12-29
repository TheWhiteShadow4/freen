const fs = require('fs');
const path = require("path");
const luabundle = require('luabundle')
const luamin = require('luamin');
const watch = require('node-watch');

const config = JSON.parse(fs.readFileSync('package.json')).build;

process.chdir(path.dirname(process.argv[1]));
console.log("location:", process.cwd());

// Pfade in denen Bibliotheken für requires gesucht werden.
const user_libs = config.libs.map(p => `${path.resolve(p)}/?.lua`);
const libs = ['?', '?.lua'].concat(user_libs);
if (process.env['LUA_PATH']) libs.push(process.env['LUA_PATH']);
const entry = (process.argv.length > 2) ? process.argv[2] : './init.lua';
const entry_path = path.resolve(entry);
process.chdir(path.dirname(entry_path));

// Bundle Datei setzen
const builddir = path.resolve("build");
const bundle_file = path.join(builddir, config.bundle);

function compile()
{
	try
	{
		if (!fs.existsSync(builddir)){
			fs.mkdirSync(builddir);
		}
		// Entfernt Zeilen, die als DEV-ONLY gekennzeichnet sind.
		var data = fs.readFileSync(entry_path, 'utf8');
		data = data.split('\n').filter(function(line){ 
			return line.indexOf("$DEV-ONLY$") == -1;
		}).join('\n')

		// Erstellt eine Bundledatei aus allen require Dateien.
		
		data = luabundle.bundleString(data, {
			paths: libs,
			metadata: config.metadata
		});
		
		if (config.minify) data = luamin.minify(data);
		fs.writeFileSync(bundle_file, data);
		
		console.log('\x1b[32m', "Compilation complete.", '\x1b[0m');
	}
	catch(err)
	{
		console.log(err);
	}
}

watch_config = {
	recursive: true, filter(f, skip) {
		// skip node_modules and build
		if (/node_modules/.test(f) || /temp/.test(f) || /build/.test(f)) return skip;
		// skip .git folder
		if (/\.git/.test(f)) return skip;
		// only watch for lua files
		return /\.lua$/.test(f);
	}}, function(evt, name) {
	console.log('%s changed.', name);
	compile();
}

compile();
console.log('\x1b[1m',"Bundle:", bundle_file, '\x1b[0m');
if (config.watch)
{
	console.log('\x1b[34m', "watching...", '\x1b[0m');
	watch('./', watch_config);
}