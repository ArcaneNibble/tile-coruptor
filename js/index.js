import '../styles/style.scss'
import * as bootstrap from 'bootstrap';
import { TileCorruptorAppInst } from '../pkg/index.js';

let rust_app_inst = undefined;

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

document.getElementById("tile_m").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.tile_minus();
});
document.getElementById("tile_p").addEventListener("click", (e) => {
    if (rust_app_inst !== undefined)
        rust_app_inst.tile_plus();
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

document
    .getElementById("filechooser")
    .addEventListener("change", choose_new_file);
