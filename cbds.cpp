#include <vector>
#include <set>
#include <unordered_map>
#include <cmath>
#include <iostream>
#include <fstream>
#include <cstdlib>
#include <unordered_map>
#include <map>
#include <utility>
#include "utility.h"

using namespace std;

typedef unsigned char BYTE;

int main(int argc, char** argv) {

    ifstream file("calibrations.txt");
    size_t in_pos;
    float in_dist;
    LookupTable lookup(2592);
    while(file >> in_pos >> in_dist) {
        lookup.addExact(in_pos, in_dist);
    }
    lookup.fill();

    // Define the placement and height of the scan line
    size_t scan_line_top;
    if(argc >= 2) scan_line_top = stoi(argv[1]);
    else scan_line_top = 1028;

    size_t scan_line_height;
    if(argc >= 3) scan_line_height = stoi(argv[2]);
    else scan_line_height = 5;

    // Defines the width of a bucket
    size_t bucket_width;
    if(argc >= 4) bucket_width = stoi(argv[3]);
    else bucket_width = 15;

    // Initialize image
    vector<BYTE> image(3 * 2592 * scan_line_height);

    // Initialize dot position
    int dot_pos = -1;

    // TODO: intialize gpio

    while(true) {
        // Take picture
        system("raspistill -o pic.bmp --nopreview -t 10 -e bmp");

        // Load the image file
        ifstream image_file("pic.bmp", std::ios::binary);

    	// Read image header into buffer
        vector<BYTE> header_buffer(54, 0);
        image_file.read((char*)(&header_buffer[0]), 54);

        // Get the image size from the buffer
        size_t width = size_t(header_buffer[19]) * 256 + size_t(header_buffer[18]);
        size_t height = size_t(header_buffer[23]) * 256 + size_t(header_buffer[22]);

        // Read image into image buffer
        image_file.seekg(54 + (height - scan_line_top - scan_line_height) * width * 3);
        image_file.read((char*)(&image[0]), image.size());

        // Run the algorithm

        // Intialize buckets and avg_pixels
        unordered_map<unsigned, Bucket> buckets;
        vector<vector<unsigned>> avg_pixels;
        for(int i = 0; i < width; i++) {
            vector<unsigned> new_pixel(3, 0);
            avg_pixels.push_back(new_pixel);
        }

        // Get average pixels and add them to buckets
        for(int i = 0; i < width; i++) {
            for(int j = 0; j < scan_line_height; j++) {
                auto color = bmp_pixel(image, width, i, j);
                avg_pixels[i][0] += color[0];
                avg_pixels[i][1] += color[1];
                avg_pixels[i][2] += color[2];
            }
            avg_pixels[i][0] /= scan_line_height;
            avg_pixels[i][1] /= scan_line_height;
            avg_pixels[i][2] /= scan_line_height;
            // cout << avg_pixels[i][0] << ", " << avg_pixels[i][1] << ", " << avg_pixels[i][2] << endl;
            unsigned main_key = i / bucket_width;
            if(buckets.find(main_key) == buckets.end()) buckets.insert(pair<unsigned, Bucket>(main_key, Bucket()));
            buckets[main_key].insert(main_key, i, simplifyColor(avg_pixels[i]));
        }
        // for(auto &ap: avg_pixels) {
        //     if(isWhite(ap)) cout << "X";
        //     else if(isRed(ap)) cout << "o";
        //     else cout << ".";
        // }
        // cout << endl;

        // Amolgomate buckets
        bool done = false;
        while(!done) {
            vector<pair<unsigned, unsigned>> keys_to_merge;

            // For each pair of different buckets, check if they are adjacent.
            // If they are, add the pair of their keys to the keys_to_merge
            for(auto &a: buckets) {
                for(auto &b: buckets) {
                    if(a.first != b.first && a.second.adjacent(b.second)) {
                        keys_to_merge.push_back(pair<unsigned, unsigned>(a.first, b.first));
                    }
                }
            }

            // For each pair of keys to merge, merge the corresponding buckets
            for(auto &pair: keys_to_merge) {
                if(buckets.find(pair.first) != buckets.find(pair.second)) {
                    Bucket a = buckets[pair.first];
                    Bucket b = buckets[pair.second];
                    buckets.erase(pair.first);
                    buckets.erase(pair.second);
                    a.merge(b);
                    unsigned new_key = a.mainKey();
                    buckets[new_key] = a;
                }
            }
            // We are done amalgomating buckets when there are no more keys to marge.
            done = keys_to_merge.empty();
        }

        // Determine dot positions
        map<unsigned, simple_color_t> final_color_positions_map;

        for(auto &b: buckets) {
            float pos_sum = 0;
            for(auto &pixel: b.second.points) pos_sum += float(pixel);
            float pos_result = pos_sum / float(b.second.points.size());
            final_color_positions_map[pos_result] = b.second.simple_color;
        }

        cout << endl;
        for(auto &fcp: final_color_positions_map) {
            cout << fcp.second << " at " << fcp.first << endl;
        }
        cout << endl;

        vector<pair<unsigned, simple_color_t>> final_color_positions;
        for(auto &fcp: final_color_positions_map) {
            final_color_positions.push_back(fcp);
        }

        // Find the dot
        unsigned dot_position = -1;

        int i = 0;
        for(auto &fcp: final_color_positions) {
            if(fcp.second == white
                && (
                    (i == 0
                        || final_color_positions[i - 1].second == red)
                    || (i == final_color_positions.size() - 1
                        || final_color_positions[i + 1].second == red)
                )
            ) {
                dot_position = fcp.first;
                break;
            }
            i++;
        }

        // if the dot was not found looking for red white red, look for just red
        for(auto &fcp: final_color_positions) {
            if(fcp.second == white) {
                dot_position = fcp.first;
                break;
            }
        }

        // if the dot was still not found looking for red, look for white
        for(auto &fcp: final_color_positions) {
            if(fcp.second == red) {
                dot_position = fcp.first;
                break;
            }
        }

        if(dot_position >= 0) {
            cout << "Dot found at x = " << dot_position << endl;
            unsigned final_pos = lookup.dist(dot_position);
            cout << "The dot is " << final_pos << " inches away" << endl;
        }
        else {
            cout << "Dot not found" << endl;
        }
    }

    return 0;
}
