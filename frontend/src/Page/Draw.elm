module Page.Draw exposing (Model, Msg, init, start, subscriptions, update, view)

{-| Draws one fixed polycube uniformly at random and shows it.
-}

import Backend exposing (DrawnPolycube)
import CellCountSlider
import Html exposing (Html, button, div, p, text)
import Html.Attributes as Attr
import Html.Events exposing (onClick)
import Scene


type alias Model =
    { cellCount : Int
    , draw : Draw
    }


type Draw
    = Drawing
    | Drawn DrawnPolycube
    | Failed String


type Msg
    = CellCountChosen String
    | DrawRequested
    | DrawFinished (Result String DrawnPolycube)


init : Model
init =
    { cellCount = CellCountSlider.initialCellCount, draw = Drawing }


{-| First draw, to request once the backend listens.
-}
start : Model -> ( Model, Cmd Msg )
start model =
    requestDraw model.cellCount


requestDraw : Int -> ( Model, Cmd Msg )
requestDraw cellCount =
    ( { cellCount = cellCount, draw = Drawing }, Backend.requestDraw cellCount )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        CellCountChosen sliderValue ->
            requestDraw (CellCountSlider.parse sliderValue |> Maybe.withDefault model.cellCount)

        DrawRequested ->
            requestDraw model.cellCount

        DrawFinished (Ok drawnPolycube) ->
            ( { model | draw = Drawn drawnPolycube }, Cmd.none )

        DrawFinished (Err reason) ->
            ( { model | draw = Failed reason }, Cmd.none )


subscriptions : Sub Msg
subscriptions =
    Backend.drawFinished DrawFinished


view : Model -> Html Msg
view model =
    div []
        [ div [ Attr.class "controls" ]
            [ CellCountSlider.view CellCountSlider.largestDrawnCellCount CellCountChosen model.cellCount
            , button [ onClick DrawRequested ] [ text "Tirer un autre" ]
            ]
        , viewDraw model.draw
        ]


viewDraw : Draw -> Html Msg
viewDraw draw =
    case draw of
        Drawing ->
            p [ Attr.class "status" ] [ text "Tirage en cours…" ]

        Drawn drawnPolycube ->
            div []
                [ p [ Attr.class "status" ]
                    [ text
                        ("Polycube n° "
                            ++ drawnPolycube.rank
                            ++ " parmi "
                            ++ drawnPolycube.polycubeCount
                            ++ " polycubes fixes"
                        )
                    ]
                , Scene.view drawnPolycube.cells
                ]

        Failed reason ->
            p [ Attr.class "status error" ] [ text ("Tirage impossible : " ++ reason) ]
