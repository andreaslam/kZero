import numpy as np
from matplotlib import pyplot

gens = [0, 1, 2, 3, 4, 5, 6, 8, 10, 20, 30, 40, 50, 56]
opponents = ["random", "greedy", "minimax-2", "minimax-4", "minimax-6"]

# wdl[gen, opponent, zero_first, 0:3]
wdl = np.array([[[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[0.1, 0.0, 0.9], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[0.2, 0.0, 0.8], [0.1, 0.0, 0.9]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[0.1, 0.0, 0.9], [0.5, 0.0, 0.5]], [[0.1, 0.0, 0.9], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[0.5, 0.0, 0.5], [0.6, 0.0, 0.4]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[0.7, 0.0, 0.3], [1.0, 0.0, 0.0]], [[0.0, 0.1, 0.9], [0.5, 0.0, 0.5]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]], [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[0.9, 0.0, 0.1], [1.0, 0.0, 0.0]], [[0.9, 0.1, 0.0], [0.9, 0.1, 0.0]], [[0.6, 0.0, 0.4], [0.7, 0.0, 0.3]], [[0.0, 0.0, 1.0], [0.1, 0.0, 0.9]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[1.0, 0.0, 0.0], [0.9, 0.0, 0.1]], [[0.6, 0.0, 0.4], [0.3, 0.2, 0.5]], [[0.3, 0.0, 0.7], [0.0, 0.0, 1.0]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[0.9, 0.1, 0.0], [0.9, 0.1, 0.0]], [[0.9, 0.0, 0.1], [0.6, 0.0, 0.4]], [[0.6, 0.0, 0.4], [0.6, 0.0, 0.4]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[0.9, 0.1, 0.0], [0.7, 0.0, 0.3]], [[0.4, 0.0, 0.6], [0.5, 0.0, 0.5]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[0.9, 0.0, 0.1], [0.9, 0.0, 0.1]], [[1.0, 0.0, 0.0], [0.8, 0.0, 0.2]], [[0.2, 0.0, 0.8], [0.3, 0.0, 0.7]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[1.0, 0.0, 0.0], [0.9, 0.0, 0.1]], [[0.9, 0.0, 0.1], [0.8, 0.0, 0.2]], [[0.5, 0.0, 0.5], [0.4, 0.0, 0.6]]], [[[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], [[0.9, 0.1, 0.0], [1.0, 0.0, 0.0]], [[1.0, 0.0, 0.0], [0.8, 0.0, 0.2]], [[0.6, 0.1, 0.3], [0.5, 0.0, 0.5]]]])

values = (wdl[:, :, :, 0] - wdl[:, :, :, 2]).mean(axis=2)
scores = (values + 1) / 2
elo = -400 * np.log10(1 / scores - 1)

pyplot.plot(gens[:10], values[:10], label=opponents)
pyplot.suptitle("Playing strength progress during training")
pyplot.title("zero visiting 1000 nodes")
pyplot.legend(title="opponent")
pyplot.ylabel("value")
pyplot.xlabel("gen")
pyplot.show()

for o in opponents:
    print(f"{o},", end="")
print()

for gi, g in enumerate(gens):
    print(f"{g},", end="")
    for oi, o in enumerate(opponents):
        print(f"{scores[gi, oi]:.3},", end="")
    print()
