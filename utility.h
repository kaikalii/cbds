#include <vector>
#include <set>
#include <unordered_map>

bool isWhite(const std::vector<unsigned int>& color);
bool isRed(const std::vector<unsigned int>& color);
bool isGreen(const std::vector<unsigned int>& color);

bool adjacentBucket(int a, int b);

std::vector<unsigned int> bmp_pixel(std::vector<unsigned char>& image, size_t width, size_t i, size_t j);
enum simple_color_t {other, green, red, white};

simple_color_t simplifyColor(const std::vector<unsigned int>& color);

class Bucket {
public:
    std::set<unsigned int> keys;
    std::set<unsigned int> points;
    simple_color_t simple_color;

    Bucket();
    unsigned int mainKey();
    void insert(unsigned int key, unsigned int point, simple_color_t sc);
    void merge(const Bucket& other);
    bool adjacent(const Bucket& other);
};

enum lookup_t {BLANK, MID, EXACT};

class Lookup {
public:
    float exact;
    std::vector<size_t> mid;
    lookup_t type;
    Lookup();
    Lookup(float e);
    Lookup(size_t a, size_t b);
};

class LookupTable {
private:
    std::vector<Lookup> v;
public:
    LookupTable(size_t n);
    float addExact(size_t i, float dist);
    float getExact(size_t i);
    void fill();
    float dist(size_t i);
};
