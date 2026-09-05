import init, { main } from "./pkg/il_compiler.js";

init().then(() => {
    main("--help");
    main("--version");
});
