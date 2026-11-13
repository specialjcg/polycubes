module Main exposing (main)

{-| Draws fixed polycubes uniformly at random and shows them in 3D.
-}

import Backend exposing (DrawnPolycube)
import Browser
import Html exposing (Html, button, div, h1, input, label, main_, p, text)
import Html.Attributes as Attr
import Html.Events exposing (onClick, onInput)
import Scene


smallestCellCount : Int
smallestCellCount =
    1


{-| Must match `MAX_INTERACTIVE_CELL_COUNT` of the backend.
-}
largestCellCount : Int
largestCellCount =
    12


initialCellCount : Int
initialCellCount =
    8


type alias Model =
    { cellCount : Int
    , draw : Draw
    }


type Draw
    = Drawing
    | Drawn DrawnPolycube
    | Failed String


type Msg
    = BackendReady
    | CellCountChosen String
    | DrawRequested
    | DrawFinished (Result String DrawnPolycube)


main : Program () Model Msg
main =
    Browser.element
        { init = \_ -> ( { cellCount = initialCellCount, draw = Drawing }, Cmd.none )
        , update = update
        , view = view
        , subscriptions = subscriptions
        }


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ Backend.ready (\_ -> BackendReady)
        , Backend.drawFinished DrawFinished
        ]


requestDraw : Int -> ( Model, Cmd Msg )
requestDraw cellCount =
    ( { cellCount = cellCount, draw = Drawing }, Backend.requestDraw cellCount )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        BackendReady ->
            requestDraw model.cellCount

        CellCountChosen sliderValue ->
            requestDraw (String.toInt sliderValue |> Maybe.withDefault model.cellCount)

        DrawRequested ->
            requestDraw model.cellCount

        DrawFinished (Ok drawnPolycube) ->
            ( { model | draw = Drawn drawnPolycube }, Cmd.none )

        DrawFinished (Err reason) ->
            ( { model | draw = Failed reason }, Cmd.none )


view : Model -> Html Msg
view model =
    main_ []
        [ h1 [] [ text "Polycubes tirés uniformément" ]
        , viewControls model.cellCount
        , viewDraw model.draw
        ]


viewControls : Int -> Html Msg
viewControls cellCount =
    div [ Attr.class "controls" ]
        [ label []
            [ text ("Cubes : " ++ String.fromInt cellCount)
            , input
                [ Attr.type_ "range"
                , Attr.min (String.fromInt smallestCellCount)
                , Attr.max (String.fromInt largestCellCount)
                , Attr.value (String.fromInt cellCount)
                , onInput CellCountChosen
                ]
                []
            ]
        , button [ onClick DrawRequested ] [ text "Tirer un autre" ]
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
