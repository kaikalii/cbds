#include <vector>
#include <set>
#include <unordered_map>
#include <cmath>
#include <iostream>
#include "utility.h"

using namespace std;

bool isWhite(const std::vector<unsigned int>& color) {
    return (color[0] + color[1] + color[2]) / 3 > 238;
}

bool isRed(const std::vector<unsigned int>& color) {
    return powf(float(color[0]), 2.0) / float(color[1] + color[2]) >= 305.f;
}

bool isGreen(const std::vector<unsigned int>& color) {
    return powf(float(color[1]), 2.0) / float(color[0] + color[2]) >= 1300.f && color[1] > 160;
}

vector<unsigned int> bmp_pixel(vector<unsigned char>& image, size_t width, size_t i, size_t j) {
    vector<unsigned int> result(3);
    result[0] = image[3 * (j * width + i) + 2];
    result[1] = image[3 * (j * width + i) + 1];
    result[2] = image[3 * (j * width + i) + 0];
    return result;
}

bool adjacentBucket(int a, int b) {
    return abs(a - b) == 1;
}

simple_color_t simplifyColor(const std::vector<unsigned int>& color) {
    if(isWhite(color)) return white;
    if(isRed(color)) return red;
    if(isGreen(color)) return green;
    return other;
}

Bucket::Bucket() :simple_color(other) {}
unsigned int Bucket::mainKey() {
    return *keys.begin();
}
void Bucket::insert(unsigned int key, unsigned int point, simple_color_t sc) {
    keys.insert(key);
    points.insert(point);
    simple_color = max(simple_color, sc);
}
void Bucket::merge(const Bucket& other) {
    keys.insert(other.keys.begin(), other.keys.end());
    points.insert(other.points.begin(), other.points.end());
}
bool Bucket::adjacent(const Bucket& other) {
    for(auto i = keys.begin(); i != keys.end(); i++) {
        for(auto j = other.keys.begin(); j != other.keys.end(); j++) {
            if(adjacentBucket(*i, *j) && simple_color == other.simple_color) return true;
        }
    }
    return false;
}

Lookup::Lookup() : type(BLANK) {}
Lookup::Lookup(float e) : exact(e), type(EXACT) {}
Lookup::Lookup(size_t a, size_t b) : type(MID) {
    mid.push_back(a);
    mid.push_back(b);
}


LookupTable::LookupTable(size_t n) : v(n) {}
float LookupTable::addExact(size_t i, float dist) {
    v[i] = Lookup(dist);
}
float LookupTable::getExact(size_t i) {
    if(v[i].type == EXACT) return v[i].exact;
    cout << "Called exact() with non-exact index: " << i << endl;
}
void LookupTable::fill() {
    for(int i = 0; i < v.size(); i++) {
        if(v[i].type == EXACT) {
            for(int j = i + 1; j < v.size(); j++) {
                if(v[j].type == EXACT) break;
                else if(v[j].type == BLANK) {
                    v[j] = Lookup(i, -1);
                }
            }
        }
    }
    for(int i = v.size() - 1; i >= 1; i--) {
        if(v[i].type == EXACT) {
            for(int j = i - 1; j >= 0; j--) {
                switch(v[j].type) {
                    case EXACT: goto tag;
                    case MID: v[j].mid[1] = i; break;
                    case BLANK: v[j] = Lookup(-1, i); break;
                }
            }
            tag:;
        }
    }
}
float LookupTable::dist(size_t i) {
    switch(v[i].type) {
        case EXACT: return v[i].exact;
        case MID: {
            size_t& a = v[i].mid[0];
            size_t& b = v[i].mid[1];
            if(a < 0 || b < 0) return -1;
            return (float(i - a) * getExact(b) + float(b - i) * getExact(a)) / (b - a);
        }
        case BLANK: {
            cout << "error, blank lookup" << endl;
			return -1;
        }
    }
}
