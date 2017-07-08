// node examples/server-example.js
var http = require('http');
var url = require('url');

function serve(ip, port)
{
        http.createServer(function (req, res) {
        	var queryData = url.parse(req.url, true).query;
        	var length = req.url.split("/")[1]
            res.writeHead(200, {'Content-Type': 'text/plain'});
            res.write(makeid(length));
            res.end("\n"+ip+":"+port);
        }).listen(port, ip);
        console.log('Server running at http://'+ip+':'+port+'/');
}

function makeid(length) {
  var text = "a";

  for (var i = 0; i < length; i++)
    text += "a";

  return text;
}

// Create three servers for
// the load balancer, listening on any
// network on the following three ports
serve('0.0.0.0', 9000);
serve('0.0.0.0', 9001);
serve('0.0.0.0', 9002);
serve('0.0.0.0', 9003);
