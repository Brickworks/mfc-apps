import io

import dash
import dash_core_components as dcc
import dash_html_components as html
import pandas as pd
import tailer as tl
import plotly.graph_objects as go
from dash.dependencies import Input, Output


app = dash.Dash(__name__,)
app.layout = html.Div(
    html.Div([
        dcc.Graph(id='live-update-graph'),
        dcc.Interval(
            id='interval-component',
            interval=1000,  # in milliseconds
            n_intervals=0)
    ]))



# Multiple components can update everytime interval gets fired.
@app.callback(Output('live-update-graph', 'figure'),
              Input('interval-component', 'n_intervals'))
def update_graph_live(n):
    fname = "support_apps/out.csv"
    file = open(fname)
    lastLines = tl.tail(file,100) #to read last 15 lines, change it  to any value.
    file.close()
    df = pd.read_csv(io.StringIO('\n'.join(lastLines)),
                     header=None,
                     names=['time',
                     'altitude_m',
                     'ascent_rate_m_s',
                     'acceleration_m_s2',
                     'lift_gas_mass_kg',
                     'ballast_mass_kg',
                     'vent_pwm',
                     'dump_pwm'])
    # df = df.set_index('time')

    fig = go.Figure()

    fig.add_trace(
        go.Scattergl(x=df['time'],
                     y=df['altitude_m'],
                     mode='markers+lines',
    ))
    return fig


if __name__ == '__main__':
    app.run_server(debug=True)
