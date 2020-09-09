let register_file_element = document.getElementById("register_file");
let memory_element = document.getElementById("memory");
let peek_address_element = document.getElementById("peek_address");
let memory_range_element = document.getElementById("memory_range");
let pc_breakpoint_list_element = document.getElementById("pc_breakpoint_list");
let new_pc_breakpoint_element = document.getElementById("pc_break");
let add_pc_breakpoint_element = document.getElementById("add_pc_breakpoint");

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

function updateMemory(peek_address) {
    let peek_window_size = 64;
    let start_address = Math.max(0, peek_address - (peek_window_size / 2));
    let end_address = Math.min(memory.length, peek_address + (peek_window_size / 2));
    memory_element.textContent = JSON.stringify(memory.slice(start_address, end_address), (key, value) => {
        if (typeof value === "number") {
            return "0x" + value.toString(16);
        }

        return value;
    });
    memory_range_element.textContent = "Start: " + start_address.toString(16) + " End: " + end_address.toString(16);
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
