import pandas as pd
import plotly.io as pio
from plotly.subplots import make_subplots
import plotly.graph_objects as go
pio.templates.default = 'plotly_white'

df = pd.read_csv("support_apps/out.csv")
df = df.set_index('time_s')

fig = make_subplots(rows=len(df.columns), cols=1)
for colindex, colname in enumerate(df.columns):
    fig.append_trace(go.Scatter(
        x=df.index,
        y=df[colname],
        name=colname,
    ), row=colindex+1, col=1)

fig.update_layout(height=2400, width=1800)
fig.write_html("out_plot.html")
