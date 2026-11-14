module Page.Enumeration exposing (Model, Msg, init, start, subscriptions, update, view)

{-| Redelmeier's enumeration, one polycube at a time: the polycube it has
built, the cells waiting to extend it, the cells it has given up so that no
polycube is reached twice, and the tally of the polycubes found per size.
-}

import Array exposing (Array)
import Backend exposing (EnumerationStep)
import CellCountSlider
import Html exposing (Html, button, div, li, p, table, td, text, th, tr, ul)
import Html.Attributes as Attr
import Html.Events exposing (onClick)
import Scene
import Time


millisecondsBetweenPolycubes : Float
millisecondsBetweenPolycubes =
    500


{-| Must match `MAX_TRACED_CELL_COUNT` of the backend.
-}
largestTracedCellCount : Int
largestTracedCellCount =
    5


initialCellCount : Int
initialCellCount =
    4


type alias Model =
    { cellCount : Int
    , trace : Trace
    }


type Trace
    = Loading
    | Walking Walk
    | Failed String


type alias Walk =
    { steps : Array EnumerationStep
    , shownStepIndex : Int
    , isPaused : Bool
    }


type Msg
    = CellCountChosen String
    | TraceFinished (Result String (List EnumerationStep))
    | ReplayRequested
    | PauseToggled
    | NextPolycubeRequested
    | PolycubeInterval


init : Model
init =
    { cellCount = initialCellCount, trace = Loading }


{-| First trace, to request once the backend listens.
-}
start : Model -> ( Model, Cmd Msg )
start model =
    requestTrace model.cellCount


requestTrace : Int -> ( Model, Cmd Msg )
requestTrace cellCount =
    ( { cellCount = cellCount, trace = Loading }
    , Backend.requestEnumerationTrace cellCount
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        CellCountChosen sliderValue ->
            requestTrace (CellCountSlider.parse sliderValue |> Maybe.withDefault model.cellCount)

        TraceFinished (Ok steps) ->
            ( { model | trace = Walking { steps = Array.fromList steps, shownStepIndex = 0, isPaused = False } }
            , Cmd.none
            )

        TraceFinished (Err reason) ->
            ( { model | trace = Failed reason }, Cmd.none )

        ReplayRequested ->
            ( changeWalk (\walk -> { walk | shownStepIndex = 0, isPaused = False }) model, Cmd.none )

        PauseToggled ->
            ( changeWalk (\walk -> { walk | isPaused = not walk.isPaused }) model, Cmd.none )

        NextPolycubeRequested ->
            ( changeWalk (showNextStep >> (\walk -> { walk | isPaused = True })) model, Cmd.none )

        PolycubeInterval ->
            ( changeWalk showNextStep model, Cmd.none )


changeWalk : (Walk -> Walk) -> Model -> Model
changeWalk change model =
    case model.trace of
        Walking walk ->
            { model | trace = Walking (change walk) }

        Loading ->
            model

        Failed _ ->
            model


showNextStep : Walk -> Walk
showNextStep walk =
    if isOnLastStep walk then
        walk

    else
        { walk | shownStepIndex = walk.shownStepIndex + 1 }


isOnLastStep : Walk -> Bool
isOnLastStep walk =
    walk.shownStepIndex + 1 >= Array.length walk.steps


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ Backend.enumerationTraceFinished TraceFinished
        , if isAdvancing model then
            Time.every millisecondsBetweenPolycubes (\_ -> PolycubeInterval)

          else
            Sub.none
        ]


isAdvancing : Model -> Bool
isAdvancing model =
    case model.trace of
        Walking walk ->
            not walk.isPaused && not (isOnLastStep walk)

        Loading ->
            False

        Failed _ ->
            False


view : Model -> Html Msg
view model =
    div []
        [ div [ Attr.class "controls" ]
            [ CellCountSlider.view largestTracedCellCount CellCountChosen model.cellCount
            , button [ onClick PauseToggled ] [ text (pauseButtonTitle model.trace) ]
            , button [ onClick NextPolycubeRequested ] [ text "Polycube suivant" ]
            , button [ onClick ReplayRequested ] [ text "Rejouer" ]
            ]
        , p [ Attr.class "explanation" ]
            [ text "Chaque polycube est prolongé par une case en attente. Les cases déjà essayées sont écartées pour tout ce qui suit : aucun polycube n'est atteint deux fois." ]
        , viewTrace model.cellCount model.trace
        ]


pauseButtonTitle : Trace -> String
pauseButtonTitle trace =
    case trace of
        Walking walk ->
            if walk.isPaused then
                "Reprendre"

            else
                "Pause"

        Loading ->
            "Pause"

        Failed _ ->
            "Pause"


viewTrace : Int -> Trace -> Html Msg
viewTrace cellCount trace =
    case trace of
        Loading ->
            p [ Attr.class "status" ] [ text "Énumération en cours…" ]

        Walking walk ->
            case Array.get walk.shownStepIndex walk.steps of
                Just step ->
                    viewStep cellCount walk step

                Nothing ->
                    p [ Attr.class "status" ] [ text "Aucun polycube à montrer." ]

        Failed reason ->
            p [ Attr.class "status error" ] [ text ("Énumération impossible : " ++ reason) ]


viewStep : Int -> Walk -> EnumerationStep -> Html Msg
viewStep cellCount walk step =
    div [ Attr.class "enumeration" ]
        [ viewLegend step
        , Scene.viewAmongNeighbours
            { largestCellCount = cellCount
            , candidates = step.candidates
            , discarded = step.discarded
            , added = []
            }
            step.placed
        , p [ Attr.class "status" ]
            [ text
                ("Polycube "
                    ++ String.fromInt (walk.shownStepIndex + 1)
                    ++ " sur "
                    ++ String.fromInt (Array.length walk.steps)
                )
            ]
        , viewCounts (List.length step.placed) step.countsBySize
        ]


viewLegend : EnumerationStep -> Html Msg
viewLegend step =
    ul [ Attr.class "legend" ]
        [ viewLegendEntry "placed" "cubes posés" (List.length step.placed)
        , viewLegendEntry "candidate" "cases en attente" (List.length step.candidates)
        , viewLegendEntry "discarded" "cases écartées" (List.length step.discarded)
        ]


viewLegendEntry : String -> String -> Int -> Html Msg
viewLegendEntry className title cellCount =
    li [ Attr.class className ] [ text (String.fromInt cellCount ++ " " ++ title) ]


{-| Tally of the polycubes found so far; the column of the polycube shown is
highlighted, since it is the one that just grew.
-}
viewCounts : Int -> List Int -> Html Msg
viewCounts shownSize countsBySize =
    let
        viewCell cell size content =
            cell [ Attr.classList [ ( "current", size == shownSize ) ] ] [ text content ]

        sizes =
            List.range 1 (List.length countsBySize)
    in
    table [ Attr.class "counts" ]
        [ tr [] (th [] [ text "Cubes" ] :: List.map (\size -> viewCell th size (String.fromInt size)) sizes)
        , tr [] (th [] [ text "Polycubes trouvés" ] :: List.map2 (\size count -> viewCell td size (String.fromInt count)) sizes countsBySize)
        ]
