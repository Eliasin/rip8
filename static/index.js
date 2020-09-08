let register_file_element = document.getElementById("register_file");
let memory_element = document.getElementById("memory");

setInterval(() => {
    let register_file_request = new XMLHttpRequest();

    register_file_request.onload = () => {
        register_file_element.textContent = register_file_request.response;
    }
    register_file_request.open("GET", "http://localhost:8000/registers");
    register_file_request.send();

    let memory_request = new XMLHttpRequest();

    memory_request.onload = () => {
        memory_element.textContent = memory_request.response;
    }
    memory_request.open("GET", "http://localhost:8000/memory");
    memory_request.send();
}, 1000);
