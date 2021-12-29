const fs = require('fs');
const path = require("path");
const luabundle = require('luabundle')
const luamin = require('luamin');
const watch = require('node-watch');
const execSync = require('child_process').execSync;


const config = JSON.parse(fs.readFileSync('package.json')).build;

process.chdir(path.dirname(process.argv[1]));
console.log("location:", process.cwd());

// Externer Pfad in dem Bibliotheken für requires gesucht werden.
const user_libs = config.libs.map(p => `${path.resolve(p)}/?.lua`);
const libs = ['?', '?.lua', process.env['LUA_PATH']].concat(user_libs);
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
			//(paths:['?', '?.lua',  `${libs}/?.lua`, process.env['LUA_PATH']],
			paths: libs,
			metadata: config.metadata
		});
		
		if (config.minify) data = luamin.minify(data);
		fs.writeFileSync(bundle_file, data);
		
		console.log("Compilation complete.")
	}
	catch(err)
	{
		console.log(err);
	}
}

compile();
console.log("Bundle:", bundle_file)
/*
watch('./', {
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
});
*/