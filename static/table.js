const fetch_table = () => {
    fetch(API + 'list/')
        .then(response => response.json())
        .then(data => {
            let table = "<table>";
            table += "<tr><th>Username</th><th>Last Match Scraped</th></tr>";
            for (i in data['trackers']) {
                let date = new Date(data['trackers'][i]['since'] * 1000);
                table += "<tr><td>" + data['trackers'][i]['name'] + "</td>"; 
                table += "<td>" + date.toLocaleString() + "</td></tr>"; 
            }
            table += "</table>";
            document.getElementById("table").innerHTML = table;
        });
}

const table_load = () => {
    document.getElementById("table").innerHTML = "<p>Loading table...</p>";
    fetch_table();
}

const init = () => {
    document.getElementById("loading").style.display = "none";
    document.getElementById("form").onsubmit = tester;
    table_load();
}
window.onload = init;
