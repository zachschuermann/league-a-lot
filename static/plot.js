const API = "http://localhost:8000/";

const tester = (e) => {
    e.preventDefault();
    let username = document.getElementById("form").elements["username"].value;
    document.getElementById("loading").style.display = "block";
    plot(username);
}

const init = () => {
    document.getElementById("loading").style.display = "none";
    document.getElementById("form").onsubmit = tester;
}
window.onload = init;

const plot = (name) => {
    fetch(API + 'matches/' + name)
        .then(response => response.json())
        .then(data => {
            plotter(data['times'], data['values']);
            document.getElementById("loading").style.display = "none";
        });
}

const plotter = (times, values) => {
    const { linear, spline, stepped, bars } = uPlot.paths;

    let data = [times, values];

    const opts = {
	    title: "League Time",
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

