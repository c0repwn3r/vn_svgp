import requests
from bs4 import BeautifulSoup

from os import listdir
from os.path import isfile, join

path = 'source'
files = [f for f in listdir(path) if isfile(join(path, f))]

for file in files:
    typ = file.split('.svg')[0].upper()
    url = f'https://contentzone.eurocontrol.int/aircraftperformance/details.aspx?ICAO={typ}&'
    page = requests.get(url)

    soup = BeautifulSoup(page.content, "html.parser")

    wingspan = soup.find(id="MainContent_wsLabelWingSpan")
    length = soup.find(id="MainContent_wsLabelLength")

    if wingspan.text == "No data" or length.text == "No data":
        print('No data for ' + typ)
        continue

    wingspan = float(wingspan.text.split(" ")[0]) * 3.28084
    length = float(length.text.split(" ")[0]) * 3.28084

    print(f'{typ} = {{ f = "{join(path,file)}", attr = "VATSIM-Radar", l = {length}, w = {wingspan}, optimizer = {{ t = "ad_floor", a_floor = 0.15, d_floor = 0.2 }}}}')