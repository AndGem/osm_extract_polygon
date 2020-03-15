# OSM Extract Polygon

This small and simple tool processes OSM pbf files to generate boundary polygons.

The main question it answers is: How do I extract the polygon of an administrative boundary?

In particular it looks for administrative boundaries (e.g., city boundaries, country boundaries, ...) and creates an output file per boundary that is in the [Osmosis Polygon format](https://wiki.openstreetmap.org/wiki/Osmosis/Polygon_Filter_File_Format).

## Usage

Execute:

```sh
osm_extract_polygon -f <INPUT_PBF_FILE>
```

The program will create a folder `<INPUT_PBF_FILE>_polygons/` in the same folder where the input file is.
This folder contains for each administrative boundary it found and extract a `.poly` file.
The name of the file is the name of the administrative boundary relation, potentially prefixed by a prefix defined in the relation under the tag `name:prefix`.

## Use Case: Extracting a smaller OSM file of a city

Assume you want to have a small OSM file of a single city.
The problem you might face is, that the smallest file you can get is still very large.
The tool [Osmosis](https://wiki.openstreetmap.org/wiki/Osmosis) can extract parts of an osm file when supplied with a [Osmosis polygon](https://wiki.openstreetmap.org/wiki/Osmosis/Polygon_Filter_File_Format) file, but you don't have such a file (and manually creating one is burdensome).

In this example I will explain how to solve this problem for the city of Karlsruhe, Germany.

#### Preparation

1. Get the newest release of `osm_extract_polygon` from the [release page](https://github.com/AndGem/osm_extract_polygon/releases).
1. Install [Osmosis](https://wiki.openstreetmap.org/wiki/Osmosis/Installation)
1. Obtain a OSM pbf file that contains Karlsruhe: Go to [geofabrik](http://download.geofabrik.de/europe/germany/baden-wuerttemberg.html) and download [Karlsruhe Regierungsbezirk](http://download.geofabrik.de/europe/germany/baden-wuerttemberg/karlsruhe-regbez-latest.osm.pbf).

#### Execution

1. Run `osm_extract_polygon`:

```sh
./osm_extract_polygon -f karlsruhe-regbez-latest.osm.pbf
```

2. Verify that the program ran, a few hundred small `*.poly` files are in the folder `karlsruhe-regbez-latest.osm.pbf_polygons/`. The file you are interested in is `Stadt_Karlsruhe.poly`.
3. Run `Osmosis`:

```sh
osmosis --read-pbf file="karlsruhe-regbez-latest.osm.pbf" --bounding-polygon file="karlsruhe-regbez-latest.osm.pbf_polygons/Stadt_Karlsruhe.poly" --write-xml file="karlsruhe.osm"
```

The output osm file you are interested in is `karlsruhe.osm`.
