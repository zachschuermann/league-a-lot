//const API = "https://leaguealot.zvs.io/";
const API = "http://localhost:8000/";

const tester = (e) => {
    e.preventDefault();
    let username = document.getElementById("form").elements["username"].value;
    document.getElementById("loading").style.display = "block";
    plot(username);
}

const plot = (name) => {
    fetch(API + 'matches/' + name)
        .then(response => response.json())
        .then(data => {
            if (data['ok']) {
                plotter(name, data['times'], data['values']);
            } else {
                alert("unable to plot " + name);
            }
            document.getElementById("loading").style.display = "none";
            let hours = data['values'].reduce((a, b) => a + b, 0) / 60;
            document.getElementById("info").innerHTML = "Found " + data['times'].length + " matches for " + name + ". Total play time is " + hours + " hours.";
            table_load();
        });
}

const plotter = (name, times, values) => {
    const { linear, spline, stepped, bars } = uPlot.paths;

    let data = [times, values];

    const opts = {
	    title: name + "'s League Time",
        ...getSize(),
	    series: [
		    {},
		    {
			    label:  "game duration",
			    stroke: "blue",
			    fill:   "rgba(0, 0, 255, 0.05)",
                paths:  bars(),
                value: (u, v) => v + " minutes",
		    },
	    ],
        axes: [
				{},
				{
					values: (u, vals) => vals.map(v => v + " min")
				}
			]
    };

    let uplot = new uPlot(opts, data, document.body);

    window.addEventListener("resize", e => {
    	uplot.setSize(getSize());
    });
}

function getSize() {
	return {
		width: window.innerWidth - 50,
		height: window.innerHeight - 200,
	}
}

