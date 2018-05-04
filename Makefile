all: cbds.o utility.o
	g++ -std=c++11 cbds.o utility.o

cbds.o: cbds.cpp
	g++ -std=c++11 -c cbds.cpp

utility.o: utility.cpp
	g++ -std=c++11 -c utility.cpp
