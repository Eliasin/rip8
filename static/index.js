let register_file_element = document.getElementById("register_file");
let memory_element = document.getElementById("memory");
let peek_address_element = document.getElementById("peek_address");
let memory_range_element = document.getElementById("memory_range");
let pc_breakpoint_list_element = document.getElementById("pc_breakpoint_list");
let new_pc_breakpoint_element = document.getElementById("pc_break");
let add_pc_breakpoint_element = document.getElementById("add_pc_breakpoint");

let peek_window_size_element = document.getElementById("peek_window_size");
let peek_window_size = 64;

peek_window_size_element.oninput = (e) => {
    if (e.target.value > 0) {
        peek_window_size = e.target.value;
    }
}

let pc_breakpoints = [];

add_pc_breakpoint_element.onclick = () => {
    let pc = new_pc_breakpoint_element.value;
    let parsed = parseInt(pc, 16);
    if (pc === "" || parsed === NaN || pc_breakpoints.includes(parsed)) {
        return;
    }

    pc_breakpoints.push(parsed);

    let add_breakpoint_request = new XMLHttpRequest();

    add_breakpoint_request.onload = () => {
        let breakpoint_element = document.createElement("li");
        let breakpoint_button = document.createElement("button");
        breakpoint_button.textContent = parsed.toString(16);
        breakpoint_button.onclick = () => {
            let delete_breakpoint_request = new XMLHttpRequest();

            delete_breakpoint_request.onload = () => {
                pc_breakpoints.filter((br) => br !== pc);
                breakpoint_element.remove();
            }

            delete_breakpoint_request.open("POST", "http://localhost:8000/delete-pc-breakpoint/" + parsed);
            delete_breakpoint_request.send();

        }
        breakpoint_element.appendChild(breakpoint_button);
        pc_breakpoint_list_element.appendChild(breakpoint_element);
    }

    add_breakpoint_request.open("POST", "http://localhost:8000/add-pc-breakpoint/" + parsed);
    add_breakpoint_request.send();
}

function formatMemoryWindow(start_address, memory_window) {
    const ENTRIES_PER_LINE = 8;
    const NUM_LINES = Math.floor(memory_window.length / ENTRIES_PER_LINE + 0.5);

    let formatted_lines = [];
    for (let i = 0; i < NUM_LINES; i++) {
        let line = memory_window.slice(i * ENTRIES_PER_LINE, (i + 1) * ENTRIES_PER_LINE);
        let line_address = start_address + i * ENTRIES_PER_LINE;
        let formatted_string = "0x" + line_address.toString(16) + ": " + JSON.stringify(line, (key, value) => {
            if (typeof value === "number") {
                return "0x" + value.toString(16);
            }

            return value;
        }) + "\n";

        formatted_lines.push(formatted_string);
    }

    return formatted_lines;
}

function updateMemory(peek_address) {
    let start_address = Math.max(0, peek_address - Math.floor(peek_window_size / 2));
    let end_address = Math.min(memory.length, peek_address + Math.floor(peek_window_size / 2));

    while(memory_element.firstChild) {
        memory_element.removeChild(memory_element.lastChild);
    }

    let formatted_memory_lines = formatMemoryWindow(start_address, memory.slice(start_address, end_address + 1));
    for (const line of formatted_memory_lines) {
        let memory_line_element = document.createElement("p");
        memory_line_element.textContent = line;

        memory_element.appendChild(memory_line_element);
    }
    memory_range_element.textContent = "Start: 0x" + start_address.toString(16) + " End: " + end_address.toString(16);
}

peek_address_element.oninput = (e) => {
    let peek_address = parseInt(e.target.value, 16);
    updateMemory(peek_address);
}

let memory = [];

setInterval(() => {
    let register_file_request = new XMLHttpRequest();

    register_file_request.onload = () => {
        register_file_element.textContent = JSON.stringify(JSON.parse(register_file_request.response), (key, value) => {
            if (typeof value === "number") {
                return "0x" + value.toString(16);
            }

            return value;
        });
    }
    register_file_request.open("GET", "http://localhost:8000/registers");
    register_file_request.send();

    let memory_request = new XMLHttpRequest();

    memory_request.onload = () => {
        memory = JSON.parse(memory_request.response);
        let peek_address = parseInt(peek_address_element.value, 16);
        updateMemory(peek_address);
    }
    memory_request.open("GET", "http://localhost:8000/memory");
    memory_request.send();
}, 1000);

let pause_button_element = document.getElementById("pause");
pause_button_element.onclick = () => {
    let pause_request = new XMLHttpRequest();

    pause_request.open("POST", "http://localhost:8000/pause");
    pause_request.send();
}

let resume_button_element = document.getElementById("resume");
resume_button_element.onclick = () => {
    let resume_request = new XMLHttpRequest();

    resume_request.open("POST", "http://localhost:8000/resume");
    resume_request.send();
}

let step_next_button_element = document.getElementById("step");
step_next_button_element.onclick = () => {
    let resume_request = new XMLHttpRequest();

    resume_request.open("POST", "http://localhost:8000/step-next");
    resume_request.send();
}
