# book-merger
Connects to Binance and Bitstamp Crypto Exchanges and merges order books to get metrics.
By default it's ethbtc pair but by running:

./book-merger-server --currencies ltcbtc 

could be made any pair.

the grpc client is: 

./book-merger-client

the web client localhost:8080:
cd ./web-trunk-client && trunk serve 
