import csv
import matplotlib.pyplot as plt
import collections

reader = csv.reader(open('rust-evc.csv', 'r'))
data = []
for row in reader:
    data.append(row)

false_positives = {}
false_negatives = {}

for row in data:
    false_positives[int(row[0])] = {}
    false_negatives[int(row[0])] = {}

for row in data:
    false_positives[int(row[0])].setdefault(int(row[1]), [])
    false_negatives[int(row[0])].setdefault(int(row[1]), [])

for row in data:
    false_positives[int(row[0])][int(row[1])].append(float(row[4].strip()))
    false_negatives[int(row[0])][int(row[1])].append(float(row[3].strip()))

for processes, data in false_positives.items():
    for prec, samples in data.items():
        false_positives[processes][prec] = sum(samples) / len(samples)

for processes, data in false_negatives.items():
    for prec, samples in data.items():
        false_negatives[processes][prec] = sum(samples) / len(samples)

plt.xlabel('Number of bits of precision (Logarithmic)')
plt.ylabel('Percentage false positives')
plt.title('Percentage false positives, Message vs Internal: 60:40')
plt.xscale('log', basex=2)

for processes, data in false_positives.items():
    sorted_data = collections.OrderedDict(sorted(data.items()))
    plt.plot(sorted_data.keys(), sorted_data.values(), 'x--', label=f'{processes} Processes')

plt.legend(loc='best')
plt.show()


reader = csv.reader(open('rust-evc.csv', 'r'))
result = {}

for row in reader:
    key, val = row
    result[key] = []

reader = csv.reader(open('rust-evc.csv', 'r'))

for row in reader:
    key, val = row
    result[key].append(val)

for key, val in result.items():
    new_val = [float(i.strip()) for i in val]
    result[key.strip(':')] = new_val

updated_result = {}

for key, val in result.items():
    updated_result[key] = val

mean_data = {}

for key, val in updated_result.items():
    mean_data[int(key)] = sum(val) / len(val)

N = 5
cumsum, moving_aves = [0], []

for i, x in enumerate(mean_data.values(), 1):
    cumsum.append(cumsum[i-1] + x)
    if i>=N:
        moving_ave = (cumsum[i] - cumsum[i-N])/N
        #can do stuff with moving_ave here
        moving_aves.append(moving_ave)

plt.xlabel('Number of events')
plt.ylabel('Size of Encoded Vector clock')
plt.title('Rate of increase of EVC size. Processes: 10')

cleaned_size = list((i for i in mean_data.keys() if i <= 320))

plt.plot(sorted(moving_aves)[:len(cleaned_size)], cleaned_size)
plt.show()
