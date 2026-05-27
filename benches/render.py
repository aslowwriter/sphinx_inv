# /// script
# dependencies = [
#   "matplotlib",
#   "pandas"
# ]
# ///


import matplotlib.pyplot as plt
import sys
import json
import pandas as pd

data_file = sys.argv[1]
output_file = sys.argv[2]

with open(data_file,'r') as fp:
    timings = json.load(fp)


names = []
mu = []
sigma = []

for impl in timings['results']:
    names.append(impl['command'])
    mu.append(impl['mean'])
    sigma.append(impl['stddev'])

df = pd.DataFrame({"name":names, "means":mu, "var":sigma}).set_index("name").reindex(["sphinx_inv (rust)","sphinx (python)"])

fig, ax = plt.subplots(figsize=(12, 6))


# Example data
hbars = ax.barh(df.index, df['means'], xerr=df['var'], align='center')
ax.invert_yaxis()
ax.set_xlabel('Time (ms)')
ax.set_title('Parsing the linux kernel documentation `objects.inv` (lower is better)')

ax.grid(which="minor", color="0.9")
ax.xaxis.grid()

ax.margins(0.2,0.1)
ax.set_xscale("log")
ax.bar_label(hbars, padding = 8, fmt=lambda s: f'{s*1000:.2f} ms')
ax.set_axisbelow(True)

plt.savefig(output_file)
