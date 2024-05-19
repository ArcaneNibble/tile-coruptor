import '../styles/style.scss'
import * as bootstrap from 'bootstrap';
import { TileCorruptorAppInst, wasm_get_builtin_graphics_codecs } from '../pkg/index.js';

let rust_app_inst = undefined;
let global_is_tiled = true;

const CODEC_HUMAN_NAMES = new Map([
    ["nes", "Tiled, 2bpp planar, non-interleaved (NES)"],
    ["gb", "Tiled, 2bpp planar, row-interleaved (GB)"],
    ["lin-1bpp-msbfirst", "Non-tiled, 1bpp, MSB->LSB"],
    ["lin-1bpp-lsbfirst", "Non-tiled, 1bpp, LSB->MSB"],

]);

let builtin_codecs = wasm_get_builtin_graphics_codecs();
let codecs_menu = document.getElementById("codecs_menu");
for (const [codec_i, codec] of builtin_codecs.entries()) {
    let name = codec.i18n_name;
    let is_tiled = codec.is_tiled;
    if (CODEC_HUMAN_NAMES.has(name))
        name = CODEC_HUMAN_NAMES.get(name);

    let a_elem = document.createElement("a");
    a_elem.classList = "dropdown-item";
    a_elem.href = "#";
    a_elem.innerText = name;

    a_elem.addEventListener("click", (e) => {
        if (rust_app_inst !== undefined) {
            rust_app_inst.change_codec(codec_i);
            document.getElementById("cur_codec").innerText = name;
            global_is_tiled = is_tiled;

            if (global_is_tiled) {
                document.getElementById("tile_pm_group").style.removeProperty("display");
                document.getElementById("px_pm_group").style.setProperty("display", "none");
            } else {
                document.getElementById("tile_pm_group").style.setProperty("display", "none");
                document.getElementById("px_pm_group").style.removeProperty("display");
            }
        }
    });

    let li_elem = document.createElement("li");
    li_elem.appendChild(a_elem);

    codecs_menu.appendChild(li_elem);

    if (codec_i == 0)
        document.getElementById("cur_codec").innerText = name;
}

async function choose_new_file(e) {
    let file = e.target.files[0];
    let file_data = await file.arrayBuffer();
    let file_data_u8 = new Uint8Array(file_data);

    if (rust_app_inst !== undefined) {
        rust_app_inst.free();
    }
    rust_app_inst = new TileCorruptorAppInst(file_data_u8);
    rust_app_inst.resize();
    rust_app_inst.render();
    rust_app_inst.update_status_bar();

    document.getElementById("open_fn").innerText = file.name;
}

document.getElementById("file_open").addEventListener("click", (e) => {
    document.getElementById("filechooser").click();
});

document.getElementById("file_export").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined) {
        let bytes = rust_app_inst.export_png();
        let blob = new Blob([bytes], { type: "image/png" });
        let url = URL.createObjectURL(blob);

        let download_elem = document.createElement("a");
        download_elem.style = "display: none;";
        download_elem.href = url;
        download_elem.download = "export.png";
        document.body.appendChild(download_elem);
        download_elem.click();
        URL.revokeObjectURL(url);
        document.body.removeChild(download_elem);
    }
});

document.getElementById("gfx_w_m").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.width_minus();
});
document.getElementById("gfx_w_p").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.width_plus();
});
document.getElementById("gfx_h_m").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.height_minus();
});
document.getElementById("gfx_h_p").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.height_plus();
});

document.getElementById("row_m").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.row_minus();
});
document.getElementById("row_p").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.row_plus();
});

document.getElementById("tile_m").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.tile_minus();
});
document.getElementById("tile_p").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.tile_plus();
});

document.getElementById("px_m").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.px_minus();
});
document.getElementById("px_p").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.px_plus();
});

document.getElementById("byte_m").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.byte_minus();
});
document.getElementById("byte_p").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.byte_plus();
});
document.getElementById("bit_m").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.bit_minus();
});
document.getElementById("bit_p").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.bit_plus();
});

document.addEventListener("keydown", (e) => {
    if (rust_app_inst === undefined)
        return;

    if (!e.altKey) {
        if (global_is_tiled) {
            if (e.key == "ArrowLeft" && !e.shiftKey) {
                rust_app_inst.tile_minus();
                e.preventDefault();
            }
            if (e.key == "ArrowRight" && !e.shiftKey) {
                rust_app_inst.tile_plus();
                e.preventDefault();
            }
        } else {
            if (e.key == "ArrowLeft" && !e.shiftKey) {
                rust_app_inst.px_minus();
                e.preventDefault();
            }
            if (e.key == "ArrowRight" && !e.shiftKey) {
                rust_app_inst.px_plus();
                e.preventDefault();
            }
        }
        if (e.key == "ArrowLeft" && e.shiftKey) {
            rust_app_inst.byte_minus();
            e.preventDefault();
        }
        if (e.key == "ArrowRight" && e.shiftKey) {
            rust_app_inst.byte_plus();
            e.preventDefault();
        }
        if (e.key == "ArrowUp") {
            rust_app_inst.row_minus(e.ctrlKey || e.metaKey);
            e.preventDefault();
        }
        if (e.key == "ArrowDown") {
            rust_app_inst.row_plus(e.ctrlKey || e.metaKey);
            e.preventDefault();
        }
    } else {
        if (e.key == "ArrowLeft") {
            rust_app_inst.width_minus();
            e.preventDefault();
        }
        if (e.key == "ArrowRight") {
            rust_app_inst.width_plus();
            e.preventDefault();
        }
        if (e.key == "ArrowUp") {
            rust_app_inst.height_minus();
            e.preventDefault();
        }
        if (e.key == "ArrowDown") {
            rust_app_inst.height_plus();
            e.preventDefault();
        }
    }
})

document.getElementById("goto_offs_btn").addEventListener("click", (e) => {
    let goto_addr_elem = document.getElementById("goto_offs_inp");
    if (!goto_addr_elem.validity.valid)
        return;
    if (rust_app_inst === undefined)
        return;
    rust_app_inst.go_to_offset(goto_addr_elem.value);
});

document
    .getElementById("filechooser")
    .addEventListener("change", choose_new_file);
