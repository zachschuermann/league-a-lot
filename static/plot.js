const API = "https://leaguealot.zvs.io/";

const tester = (e) => {
    e.preventDefault();
    let username = document.getElementById("form").elements["username"].value;
    document.getElementById("loading").style.display = "block";
    plot(username);
}

var dots = 0;
const loadingDots = () => {
    let s = "<p>Loading";
    for (i = 0; i < dots; i++) {
        s += ".";
    }
    dots += 1;
    if (dots > 3) { dots = 0; }
    s += "</p>";
    document.getElementById("loading").innerHTML = s;
}

const plot = (name) => {
    let interval = setInterval(loadingDots, 400);
    fetch(API + 'matches/' + name)
        .then(response => response.json())
        .then(data => {
            if (data['ok']) {
                plotter(name, data['times'], data['values']);
            } else {
                alert("unable to plot " + name);
            }
            document.getElementById("loading").style.display = "none";
            clearInterval(interval);
            let hours = data['values'].reduce((a, b) => a + b, 0) / 60;
            document.getElementById("info").innerHTML = "Found " + data['times'].length + " matches for " + name + ". Total play time is " + hours + " hours.";
            table_load();
        });
}

const plotter = (name, times, values) => {
    const { linear, spline, stepped, bars } = uPlot.paths;

    const seriesStroke = darkMode ? "#59b7ff" : "#0079d6";
    const axesStroke = darkMode ? "#f5f5f5" : "#050505";
    const gridStroke = darkMode ? "#272b2e" : "#ebebeb";

    let data = [times, values];

    const opts = {
	    title: name + "'s League Time",
        ...getSize(),
	    series: [
		    {},
		    {
			    label:  "game duration",
			    stroke: seriesStroke,
			    fill:   "rgba(0, 0, 255, 0.05)",
                paths:  bars(),
                value: (u, v) => v + " minutes",
		    },
	    ],
        axes: [
				{
                    stroke: axesStroke,
                    grid: {
                        stroke: gridStroke,
                        width: 1,
                    },
                    ticks: {
                        stroke: gridStroke,
                        width: 1,
                    }
                },
				{
                    stroke: axesStroke,
					values: (u, vals) => vals.map(v => v + " min"),
                    grid: {
                        stroke: gridStroke,
                        width: 1,
                    },
                    ticks: {
                        stroke: gridStroke,
                        width: 1,
                    }
				}
			]
    };

    let uplot = new uPlot(opts, data, document.getElementById("plot"));

    window.addEventListener("resize", e => {
    	uplot.setSize(getSize());
    });
}

function getSize() {
	return {
		width: window.innerWidth - 50,
		height: window.innerWidth / 3,
	}
}

