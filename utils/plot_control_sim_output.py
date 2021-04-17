import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv("support_apps/out.csv")

fig, ax1 = plt.subplots()
ax1.plot(df['t'], df['alt'], label="altitude", color="tab:red")
ax1.set_xlabel("Time (s)")
ax1.set_ylabel("Altitude (m)")
#ax1.legend()


ax2 = ax1.twinx()
ax2.set_ylabel("PWM amount")
ax2.plot(df['t'], df['dump'], label="dump")
ax2.plot(df['t'], df['vent'], label="vent")
#ax2.legend()

lines = []
labels = []

for ax in fig.axes:
    # ax.set_xlim([0, 3000])
    axLine, axLabel = ax.get_legend_handles_labels()
    lines.extend(axLine)
    labels.extend(axLabel)

    
fig.legend(lines, labels, "upper left")

#plt.tight_layout()
plt.savefig("t.png")

