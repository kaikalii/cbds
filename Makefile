all: cbds.o utility.o
	g++ -std=c++11 cbds.o utility.o -lwiringPi

cbds.o: cbds.cpp
	g++ -std=c++11 -c cbds.cpp -lwiringPi

utility.o: utility.cpp
	g++ -std=c++11 -c utility.cpp
