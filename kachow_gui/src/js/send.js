const discover = document.getElementById("Discover");
const sendFiles = document.getElementById("SendFiles");
let FilesToSend = [];
function loadDiscoverDevices(devices) {
    function newDiscoverDevices(deviceID, deviceName, deviceImage) {
        let button = document.createElement("button");
        let img = document.createElement("div");
        img.classList.add("img")
        img.style.backgroundImage = `url('${deviceImage}')`;
        let span = document.createElement("span");
        span.textContent = deviceName;
        button.appendChild(img);
        button.appendChild(span);
        button.addEventListener("click", () => {
            console.log(deviceID);
        })
        discover.appendChild(button);
    }

    discover.innerHTML = "";
    devices.forEach(device => {
        newDiscoverDevices(device.id, dilevice.name, device.image);

    });
}

function loadSendFiles(files) {
    function newFile(path) {
        function getIcon(name) {
            let name_split = name.split(".");
            let ext = name_split[name_split.length - 1];
            if ([
                "webp", "jpg", "jpeg", "png", "gif", "avif", "svg", "tiff", "tif", "bmp", "ico", "heic", "heif", "raw", "cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "psd", "ai", "eps", "xcf", "indd", "sketch", "fig", "fits", "dcm", "exr", "hdr", "tga"].includes(ext)) {
                return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h560q33 0 56.5 23.5T840-760v560q0 33-23.5 56.5T760-120H200Zm40-160h480L570-480 450-320l-90-120-120 160Z"/></svg>`;
            } else if ([
                "html", "htm", "xhtml", "php", "phtml", "asp", "aspx", "jsp", "twig", "blade.php", "liquid", "handlebars", "hbs", "mustache", "astro", "vue", "svelte", "marko", "haml", "slim", "pug", "jade", "njk", "ejs", "eta", "jinja", "jinja2", "j2", "vsl", "vtl", "xml", "xsl", "xslt", "xsd", "svg", "rss", "atom", "plist", "wsdl", "xaml", "kml", "gpx", "svgz", "axml", "opml", "mxml", "wxs", "wxi", "xul", "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx", "wasm", "coffee", "litcoffee", "dart", "actionscript", "as", "c", "h", "cpp", "cxx", "cc", "cp", "c++", "hpp", "hxx", "hh", "h++", "cs", "csx", "java", "jav", "class", "kt", "kts", "ktm", "rs", "go", "swift", "m", "mm", "zig", "nim", "nims", "d", "di", "pas", "pp", "inc", "lpr", "fortran", "f", "for", "f90", "f95", "f03", "f08", "f77", "ada", "adb", "ads", "cob", "cbl", "vhd", "vhdl", "v", "sv", "svh", "vpp", "sol", "vy", "py", "pyw", "pyt", "pyi", "pyx", "pxd", "pxi", "rb", "rbw", "rake", "gemspec", "podspec", "perl", "pl", "pm", "t", "ph", "lua", "r", "rmd", "rdata", "rds", "jl", "ex", "exs", "erl", "hrl", "clj", "cljs", "cljc", "edn", "hs", "lhs", "elm", "ml", "mli", "sml", "mly", "mll", "fs", "fsi", "fsx", "fsscript", "groovy", "gvy", "gy", "gsh", "scala", "sc", "tcl", "tk", "lisp", "lsp", "l", "cl", "fasl", "scm", "ss", "rkt", "rktl", "re", "rei", "odin", "v", "vsh", "ha", "gleam", "janet", "wren", "pony", "hy", "io", "pike", "chpl", "mako", "sh", "bash", "zsh", "fish", "ksh", "csh", "tcsh", "bat", "cmd", "ps1", "psm1", "psd1", "ps1xml", "vbs", "vbe", "wsf", "wsc", "awk", "sed", "nu", "elv", "ion", "xonsh", "asm", "s", "a51", "inc", "nasm", "masm", "yasm", "fasm"
            ].includes(ext)) {
                return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="M320-240 80-480l240-240 57 57-184 184 183 183-56 56Zm320 0-57-57 184-184-183-183 56-56 240 240-240 240Z"/></svg>`;
            } else if ([
                "sql", "psql", "tsql", "plsql", "pls", "plb", "sqlite", "sqlite3", "db", "db3", "s3db", "sl3", "mdb", "accdb", "accde", "accdr", "dbf", "fdb", "gdb", "ib", "myd", "myi", "frm", "ibd", "mdf", "ldf", "ndf", "ora", "dmp", "dump", "cql", "cqldataprep", "graphql", "gql", "neo4j", "cypher", "aql", "influx", "prql", "edgeql", "hql", "pql", "surrealql", "surql", "kql", "linq"
            ].includes(ext)) {
                return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="M735-567q105-47 105-113T735-793q-105-47-255-47t-255 47q-105 47-105 113t105 113q105 47 255 47t255-47ZM582.5-428.5Q644-437 701-456t98-49.5q41-30.5 41-74.5v100q0 44-41 74.5T701-356q-57 19-118.5 27.5T480-320q-41 0-102.5-8.5T259-356q-57-19-98-49.5T120-480v-100q0 44 41 74.5t98 49.5q57 19 118.5 27.5T480-420q41 0 102.5-8.5Zm0 200Q644-237 701-256t98-49.5q41-30.5 41-74.5v100q0 44-41 74.5T701-156q-57 19-118.5 27.5T480-120q-41 0-102.5-8.5T259-156q-57-19-98-49.5T120-280v-100q0 44 41 74.5t98 49.5q57 19 118.5 27.5T480-220q41 0 102.5-8.5Z"/></svg>`;
            } else if (["css", "scss", "sass", "less", "styl", "stylus", "postcss", "pcss", "sss", "gss", "mss", "tss", "hss", "qss", "wxss", "acss", "uvss", "theme", "styles"].includes(ext)) {
                return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="M420-360q-17 0-28.5-11.5T380-400v-40h60v20h80v-40H420q-17 0-28.5-11.5T380-500v-60q0-17 11.5-28.5T420-600h120q17 0 28.5 11.5T580-560v40h-60v-20h-80v40h100q17 0 28.5 11.5T580-460v60q0 17-11.5 28.5T540-360H420Zm260 0q-17 0-28.5-11.5T640-400v-40h60v20h80v-40H680q-17 0-28.5-11.5T640-500v-60q0-17 11.5-28.5T680-600h120q17 0 28.5 11.5T840-560v40h-60v-20h-80v40h100q17 0 28.5 11.5T840-460v60q0 17-11.5 28.5T800-360H680Zm-520 0q-17 0-28.5-11.5T120-400v-160q0-17 11.5-28.5T160-600h120q17 0 28.5 11.5T320-560v40h-60v-20h-80v120h80v-20h60v40q0 17-11.5 28.5T280-360H160Z"/></svg>`;
            }
            else if (["json", "json5", "jsonc", "jsonl", "ndjson", "geojson", "topojson", "mjsn", "hjson", "bson", "cbor", "ubjson", "msgpack", "ion", "yaml", "yml", "toml", "ron", "hcl", "tf", "tfvars", "proto", "protobuf", "flatbuffers", "fbs", "thrift", "avro", "asn1", "asn", "mpack"].includes(ext)) {
                return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="M580-126v-125h76q22 0 37.5-15.5T709-305v-50q0-40 22.5-72t60.5-46v-14q-38-14-60.5-46T709-605v-51q0-22-15.5-37.5T656-709h-76v-125h109q60 0 102.5 42.5T834-688v50q0 22 15.5 38t38.5 16h26v208h-26q-23 0-38.5 15.5T834-323v51q0 61-42.5 103.5T689-126H580Zm-309 0q-60 0-102.5-42.5T126-272v-51q0-22-15.5-37.5T73-376H46v-208h27q22 0 37.5-16t15.5-38v-50q0-61 42.5-103.5T271-834h109v125h-75q-22 0-38 15.5T251-656v51q0 40-22.5 72T169-487v14q37 14 59.5 46t22.5 72v50q0 23 16 38.5t38 15.5h75v125H271Z"/></svg>`;
            }
            else if (["mp4", "m4v", "m4p", "mkv", "webm", "mov", "qt", "avi", "wmv", "asf", "flv", "f4v", "f4p", "f4a", "f4b", "vob", "ogv", "ogg", "drc", "gifv", "mng", "mts", "m2ts", "ts", "tsv", "3gp", "3g2", "m2v", "m4v", "svi", "roq", "nsv", "rm", "rmvb", "viv", "yuv", "amv", "mpg", "mpeg", "mpe", "mpv", "m2v", "mod", "tod"].includes(ext)) {
                return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="m172-834 80 160h120l-80-160h80l80 160h120l-80-160h80l80 160h120l-80-160h96q53 0 89.5 36.5T914-708v456q0 53-36.5 89.5T788-126H172q-53 0-89.5-35.5T46-249v-459q0-53 36.5-89.5T172-834Z"/></svg>`;
            }
            else if (["mp3", "wav", "ogg", "oga", "m4a", "aac", "flac", "alac", "wma", "aiff", "aif", "aifc", "opus", "mid", "midi", "kar", "rmi", "amr", "3ga", "awb", "ac3", "eac3", "dts", "dtshd", "pcm", "raw", "ra", "rm", "mka", "caf", "voc", "au", "snd", "gsm", "m3u", "m3u8", "pls", "cda"].includes(ext)) {
                return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="M255-215v-530h111v530H255ZM425-46v-868h110v868H425ZM86-385v-190h111v190H86Zm507 170v-530h111v530H593Zm170-170v-190h111v190H763Z"/></svg>`;
            } else if (["ini", "conf", "config", "cfg", "properties", "env", "env.local", "env.development", "env.production", "dotfile", "rc", "bashrc", "zshrc", "vimrc", "editorconfig", "prettierrc", "eslintrc", "babelrc", "npmrc", "yarnrc", "gitignore", "gitattributes", "dockerignore", "dockerfile", "inf", "cnf", "reg", "sys", "manifest", "plist", "mobileprovision", "gradle", "properties", "lock", "workspace", "prefs"].includes(ext)) {
                return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="m341-46-18-139q-5-2-10-5l-10-6-129 55L34-385l111-84v-22L34-575l140-242 131 54q5-3 9-5.5t9-4.5l18-141h278l18 141q5 2 10 4.5t10 5.5l129-54 140 242-112 84v11q0 3-.5 5.5t-.5 5.5l112 84-141 244-129-55q-5 3-9 6t-9 5L619-46H341Zm138-294q58 0 99-41t41-99q0-58-41-99t-99-41q-58 0-99 41t-41 99q0 58 41 99t99 41Z"/></svg>`;
            } else if (["zip", "rar", "7z", "tar", "gz", "gzip", "tgz", "bz2", "bzip2", "tbz", "tbz2", "xz", "txz", "lz", "lzma", "lz4", "tlz", "z", "taz", "iso", "img", "vcd", "cab", "arj", "ace", "lzh", "lha", "zoo", "cpio", "shar", "sit", "sitx", "sea", "dmg", "pkg", "deb", "rpm", "apk", "xapk", "jar", "war", "ear", "a", "ar", "paq", "zpaq", "pea", "wim", "swm", "esd", "zz", "zst", "tzst"].includes(ext)) {
                return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960" ><path d="M172-126q-53 0-89.5-36.5T46-252v-456q0-53 36.5-89.5T172-834h213l95 95h308q53 0 89.5 36.5T914-613v361q0 53-36.5 89.5T788-126H172Zm393-126h75v-75.5h75V-403h-75v-74.67h75v-74.66h-75V-628h-75v75.17h75v75.16h-75V-403h75v75.5h-75v75.5Z"/></svg>`;
            }

            return `<svg xmlns="http://www.w3.org/2000/svg"  viewBox="0 -960 960 960"><path d="M252-46q-53 0-89.5-36.5T126-172v-616q0-53 36.5-89.5T252-914h322l260 260v482q0 53-36.5 89.5T708-46H252Zm256-542h200L508-788v200Z"/></svg>`
        }
        let path_split = path.split("/");
        if (path_split == [path]) {
            path_split = path.split("\\");
        }
        let name = path_split[path_split.length - 1];
        let div = document.createElement("div");
        div.classList.add("file");
        let span = document.createElement("span");
        span.textContent = name;
        let button = document.createElement("button");
        button.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"/></svg>`;
        button.addEventListener("click", () => {
            FilesToSend = FilesToSend.filter(p => p !== path);
            loadSendFiles(FilesToSend);
        })

        div.innerHTML = getIcon(name);
        div.appendChild(span);
        div.appendChild(button);
        sendFiles.appendChild(div);

    }

    sendFiles.innerHTML = "";
    files.forEach(path => {
        newFile(path)
    });


}




(async () => {
  const appWindow = window.__TAURI__.window.getCurrentWindow();

  await appWindow.onDragDropEvent((event) => {
    menuSend.click();
    const absolutePaths = event.payload.paths;
      absolutePaths.forEach((path) => {
        if (!FilesToSend.includes(path)) { 
            FilesToSend.push(path);
            loadSendFiles(FilesToSend);
        }
      });
    });
})()