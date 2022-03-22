import io

from dash import Dash, html, dcc
from dash.dependencies import Input, Output
import pandas as pd
import plotly.graph_objects as go
import tailer as tl

DEFAULT_DATA_CSV = 'cli/out.csv'
DEFAULT_TAIL_LINES = 1000
PLOT_TEMPLATE = 'plotly_white'

# -----------------------------------------------------------------------------
# Page Layout
# -----------------------------------------------------------------------------
app = Dash(__name__, )
app.layout = html.Div([
    html.Div([
        # dashboard setup
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
                             'label': '5000 samples',
                             'value': 5000
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
    ]),
    html.Div(
        [
            html.Div([
                # motion
                dcc.Graph(id='altitude'),
                dcc.Graph(id='ascent_rate'),
                dcc.Graph(id='acceleration'),
            ]),
            html.Div([
                # vent status
                dcc.Graph(id='lift_gas_mass'),
                dcc.Graph(id='vent_pwm'),
                dcc.Graph(id='atmo_pres'),
            ]),
            html.Div([
                # dump status
                dcc.Graph(id='ballast_mass'),
                dcc.Graph(id='dump_pwm'),
                dcc.Graph(id='atmo_temp'),
            ])
        ],
        style={
            'display': 'flex',
            'flex-direction': 'row'
        }),
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
        file, n_lines)  # to read last n lines, change it to any value.
    file.close()
    return pd.read_csv(io.StringIO('\n'.join(lastLines)),
                       header=1,
                       names=[
                           'time_s',
                           'altitude_m',
                           'ascent_rate_m_s',
                           'acceleration_m_s2',
                           'lift_gas_mass_kg',
                           'ballast_mass_kg',
                           'vent_pwm',
                           'dump_pwm',
                           'gross_lift_N',
                           'free_lift_N',
                           'atmo_temp_K',
                           'atmo_pres_Pa',
                       ]).to_json()


def simple_scatter(df, name):
    fig = go.Figure()
    fig.update_layout(
        title=name,
        xaxis_title='Time (s)',
        template=PLOT_TEMPLATE,
    )
    fig.add_trace(
        go.Scattergl(x=df['time_s'],
                     y=df[name],
                     mode='markers+lines',
                     name=name))
    return fig


# Multiple components can update everytime interval gets fired.
@app.callback(
    Output('altitude', 'figure'),
    Input('interval-component', 'n_intervals'),
    Input('last-n-lines', 'data'),
)
def altitude(interval, data):
    df = pd.read_json(data)
    return simple_scatter(df, 'altitude_m')


@app.callback(
    Output('ascent_rate', 'figure'),
    Input('interval-component', 'n_intervals'),
    Input('last-n-lines', 'data'),
)
def ascent_rate(interval, data):
    df = pd.read_json(data)
    return simple_scatter(df, 'ascent_rate_m_s')


@app.callback(
    Output('acceleration', 'figure'),
    Input('interval-component', 'n_intervals'),
    Input('last-n-lines', 'data'),
)
def acceleration(interval, data):
    df = pd.read_json(data)
    return simple_scatter(df, 'acceleration_m_s2')


@app.callback(
    Output('lift_gas_mass', 'figure'),
    Input('interval-component', 'n_intervals'),
    Input('last-n-lines', 'data'),
)
def lift_gas_mass(interval, data):
    df = pd.read_json(data)
    return simple_scatter(df, 'lift_gas_mass_kg')


@app.callback(
    Output('ballast_mass', 'figure'),
    Input('interval-component', 'n_intervals'),
    Input('last-n-lines', 'data'),
)
def ballast_mass(interval, data):
    df = pd.read_json(data)
    return simple_scatter(df, 'ballast_mass_kg')


@app.callback(
    Output('vent_pwm', 'figure'),
    Input('interval-component', 'n_intervals'),
    Input('last-n-lines', 'data'),
)
def vent_pwm(interval, data):
    df = pd.read_json(data)
    return simple_scatter(df, 'vent_pwm')


@app.callback(
    Output('dump_pwm', 'figure'),
    Input('interval-component', 'n_intervals'),
    Input('last-n-lines', 'data'),
)
def dump_pwm(interval, data):
    df = pd.read_json(data)
    return simple_scatter(df, 'dump_pwm')


@app.callback(
    Output('atmo_temp', 'figure'),
    Input('interval-component', 'n_intervals'),
    Input('last-n-lines', 'data'),
)
def atmo_temp(interval, data):
    df = pd.read_json(data)
    return simple_scatter(df, 'atmo_temp_K')


@app.callback(
    Output('atmo_pres', 'figure'),
    Input('interval-component', 'n_intervals'),
    Input('last-n-lines', 'data'),
)
def atmo_pres(interval, data):
    df = pd.read_json(data)
    return simple_scatter(df, 'atmo_pres_Pa')


if __name__ == '__main__':
    app.run_server(debug=True)
