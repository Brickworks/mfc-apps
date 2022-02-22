import io

from dash import Dash
from dash.dependencies import Input, Output
import dash_core_components as dcc
import dash_html_components as html
import pandas as pd
import plotly.graph_objects as go
import tailer as tl

DEFAULT_DATA_CSV = 'support_apps/out.csv'
DEFAULT_TAIL_LINES = 1000
PLOT_TEMPLATE = 'plotly_white'

# -----------------------------------------------------------------------------
# Page Layout
# -----------------------------------------------------------------------------
app = Dash(__name__, )
app.layout = html.Div([
    dcc.Dropdown(id='tail_lines',
                 options=[
                     {
                         'label': '100 samples',
                         'value': 100
                     },
                     {
                         'label': '1000 samples',
                         'value': 1000
                     },
                     {
                         'label': '10000 samples',
                         'value': 10000
                     },
                 ],
                 value=1000),
    dcc.Interval(
        id='interval-component',
        interval=1000,  # in milliseconds
        n_intervals=0),
    # store the last N data points
    dcc.Store(id='last-n-lines'),
    dcc.Graph(id='altitude'),
])


# -----------------------------------------------------------------------------
# Components
# -----------------------------------------------------------------------------
@app.callback(Output('last-n-lines', 'data'),
              Input('interval-component', 'n_intervals'),
              Input('tail_lines', 'value'))
def get_last_n_lines_from_csv(intervals, n_lines, fname=DEFAULT_DATA_CSV):
    file = open(fname)
    lastLines = tl.tail(
        file, n_lines)  #to read last n lines, change it to any value.
    file.close()
    return pd.read_csv(io.StringIO('\n'.join(lastLines)),
                       header=None,
                       names=[
                           'time_s', 'altitude_m', 'ascent_rate_m_s',
                           'acceleration_m_s2', 'lift_gas_mass_kg',
                           'ballast_mass_kg', 'vent_pwm', 'dump_pwm'
                       ]).to_json()


# Multiple components can update everytime interval gets fired.
@app.callback(
    Output('altitude', 'figure'),
    Input('interval-component', 'n_intervals'),
    Input('last-n-lines', 'data'),
)
def altitude(interval, data):
    df = pd.read_json(data)
    fig = go.Figure()
    fig.update_layout(
        title='Altitude (m)',
        xaxis_title='Time (s)',
        template=PLOT_TEMPLATE,
    )
    fig.add_trace(
        go.Scattergl(x=df['time_s'],
                     y=df['altitude_m'],
                     mode='markers+lines',
                     name='altitude_m'))
    return fig


if __name__ == '__main__':
    app.run_server(debug=True)
