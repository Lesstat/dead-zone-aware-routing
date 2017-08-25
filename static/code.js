var map = L.map('mapid', { closePopupOnClick: false}).setView([48.7456643, 9.1070856], 15);
var start = true;

var startPopup = L.popup({ autoClose: false });
var endPopup = L.popup({ autoClose: false });
var geoJson = L.layerGroup([]).addTo(map);
var towerLayer = L.layerGroup([]).addTo(map);

L.tileLayer('http://{s}.tile.openstreetmap.org/{id}/{z}/{x}/{y}.png', {
    maxZoom: 18,
    attribution: 'Map data &copy; <a href="http://openstreetmap.org">OpenStreetMap</a> contributors, ' +
	'<a href="http://creativecommons.org/licenses/by-sa/2.0/">CC-BY-SA</a>, ',
    id: ''
}).addTo(map);

$("input").change(function() {
    calcDistWithCurrentSelection();
});

function calcDistWithCurrentSelection(){
    var goal = document.querySelector('input[name="goal"]:checked').value;
    var move = document.querySelector('input[name="move"]:checked').value;
    var provider = document.querySelector('input[name="provider"]:checked').value;
    geoJson.clearLayers(); 
    if(provider == "all"){
	calcDist(goal, move, "telekom");
	calcDist(goal, move, "vodafone");
	calcDist(goal, move, "o2");
	calcDist(goal, move, "none");
    }else {
	calcDist(goal, move, provider);
    }
}

function onMapClick(e) {
    var id;
    id = "start";
    startPopup.setLatLng(e.latlng).setContent("Start at " + e.latlng.toString()).addTo(map);
    getNode(id, e.latlng) ;
}

function onRightClick(e){
    var id = "end";
    endPopup.setLatLng(e.latlng).setContent("End at " + e.latlng.toString()).addTo(map);
    getNode(id, e.latlng) ;
}


map.on('click', onMapClick);
map.on('contextmenu', onRightClick);

function getNode(id, latlng){
    
    var xmlhttp = new XMLHttpRequest();
    
    xmlhttp.onload = function() {
	if (xmlhttp.status == 200) {
	    document.getElementById(id).innerHTML = xmlhttp.responseText;
	    calcDistWithCurrentSelection();
	}
    };
    xmlhttp.open("GET", "/node_at?lat="+ latlng.lat  + "&long=" + latlng.lng, true);
    xmlhttp.send();
}

function calcDist(goal, move, provider){
    
    var xmlhttp = new XMLHttpRequest();
    
    xmlhttp.responseType = 'json';
    xmlhttp.onload = function() {
	if (xmlhttp.status == 200) {
	    var myStyle = {
		"color": getColor(provider),
		"weight": 5,
		"opacity": 0.65
	    };
	    document.getElementById("dist").innerHTML = xmlhttp.response.distance;
	    document.getElementById("time").innerHTML = xmlhttp.response.travel_time;
	    geoJson.addLayer(L.geoJSON(xmlhttp.response.route.geometry, { style: myStyle }));
	}
	else {
	    document.getElementById("dist").innerHTML = "Unkown";
	}
    };
    var provider_param ="";
    if (provider != "none"){
	provider_param = "&provider=" + provider;
    }
    var s = document.getElementById("start").innerHTML;
    var t = document.getElementById("end").innerHTML;
    xmlhttp.open("GET", "/route?s="+ s  + "&t=" + t+ "&goal=" + goal + "&move=" + move + provider_param, true);
    xmlhttp.send();
}

function renderTowers(){
    
    var xmlhttp = new XMLHttpRequest();
    
    console.log("loading towers");
    
    var provider = document.querySelector('input[name="provider"]:checked').value;
    xmlhttp.responseType = 'json';
    xmlhttp.onload = function() {
	if (xmlhttp.status == 200) {
	    towerLayer.clearLayers();
	    var col = getColor(provider);
	    xmlhttp.response.forEach(function(item, index, array) {
		towerLayer.addLayer(L.circle([item.lat, item.lon], { radius: item.range * 1000, color: col, fillOpacity: 0.05 , opacity: 0.2, weight: 1} ));
	    });
	}
    };
    var bounds = map.getBounds();
    var latMin = bounds.getSouth();
    var latMax = bounds.getNorth();
    var longMin = bounds.getWest();
    var longMax = bounds.getEast();
    xmlhttp.open("GET", "/towers?lat_min="+ latMin + "&lat_max=" + latMax + "&lon_min="+ longMin + "&lon_max=" + longMax + "&provider=" + provider, true);
    xmlhttp.send();
}

function getColor(provider){
    switch (provider){
    case "telekom":
	return "#E20074";
    case "vodafone":
	return "#E60000";
    case "o2":
	return "#0090D0";
    }
    return "#000000";
}
