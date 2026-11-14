module Page.Automaton exposing (Model, Msg, init, start, subscriptions, update, view)

{-| A cellular automaton in three dimensions, seeded with a polycube: at
each generation a cell lives on, dies or is born according to its number of
living neighbours. The page plays the generations of one seed and can count
what becomes of every polycube of a size.
-}

import Backend exposing (AutomatonCensus, AutomatonRun, Fate(..))
import CellCountSlider
import Html exposing (Html, button, div, h2, p, section, span, text)
import Html.Attributes as Attr
import Html.Events exposing (onClick)
import Scene
import Time


{-| Must match `MAX_SEED_CELL_COUNT` of the backend.
-}
largestSeedCellCount : Int
largestSeedCellCount =
    9


{-| Must match `MAX_CENSUS_CELL_COUNT` of the backend.
-}
largestCensusCellCount : Int
largestCensusCellCount =
    6


initialCellCount : Int
initialCellCount =
    6


millisecondsBetweenGenerations : Float
millisecondsBetweenGenerations =
    600


type SeedKind
    = Uniform
    | Growth
    | MostLooped


type Rule
    = Bays4555
    | Bays5766
    | Faces2323


type alias Model =
    { cellCount : Int
    , seedKind : SeedKind
    , rule : Rule
    , run : Run
    , census : Census
    }


type Run
    = Running
    | Ran { run : AutomatonRun, shownGeneration : Int }
    | RunFailed String


type Census
    = NotCounted
    | Counting
    | Counted AutomatonCensus
    | CensusFailed String


type Msg
    = CellCountChosen String
    | SeedKindChosen SeedKind
    | RuleChosen Rule
    | RunRequested
    | RunFinished (Result String AutomatonRun)
    | GenerationInterval
    | CensusRequested
    | CensusFinished (Result String AutomatonCensus)


init : Model
init =
    { cellCount = initialCellCount
    , seedKind = Uniform
    , rule = Bays4555
    , run = Running
    , census = NotCounted
    }


{-| First run, to request once the backend listens.
-}
start : Model -> ( Model, Cmd Msg )
start model =
    requestRun model


{-| A new seed or rule makes the previous census stale.
-}
requestRun : Model -> ( Model, Cmd Msg )
requestRun model =
    ( { model | run = Running, census = NotCounted }
    , Backend.requestAutomatonRun
        { cellCount = model.cellCount
        , seedKind = seedKindName model.seedKind
        , rule = ruleName model.rule
        }
    )


{-| Names of the backend's `SeedKind`.
-}
seedKindName : SeedKind -> String
seedKindName seedKind =
    case seedKind of
        Uniform ->
            "uniform"

        Growth ->
            "growth"

        MostLooped ->
            "mostLooped"


{-| Names of the backend's `RulePreset`.
-}
ruleName : Rule -> String
ruleName rule =
    case rule of
        Bays4555 ->
            "bays4555"

        Bays5766 ->
            "bays5766"

        Faces2323 ->
            "faces2323"


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        CellCountChosen sliderValue ->
            requestRun
                { model | cellCount = CellCountSlider.parse sliderValue |> Maybe.withDefault model.cellCount }

        SeedKindChosen seedKind ->
            requestRun { model | seedKind = seedKind }

        RuleChosen rule ->
            requestRun { model | rule = rule }

        RunRequested ->
            requestRun model

        RunFinished (Ok run) ->
            ( { model | run = Ran { run = run, shownGeneration = 0 } }, Cmd.none )

        RunFinished (Err reason) ->
            ( { model | run = RunFailed reason }, Cmd.none )

        GenerationInterval ->
            ( { model | run = showNextGeneration model.run }, Cmd.none )

        CensusRequested ->
            ( { model | census = Counting }
            , Backend.requestAutomatonCensus
                { cellCount = model.cellCount, rule = ruleName model.rule }
            )

        CensusFinished (Ok census) ->
            ( { model | census = Counted census }, Cmd.none )

        CensusFinished (Err reason) ->
            ( { model | census = CensusFailed reason }, Cmd.none )


{-| After the last generation the seed comes back: the run plays in a loop.
-}
showNextGeneration : Run -> Run
showNextGeneration run =
    case run of
        Ran played ->
            Ran
                { played
                    | shownGeneration =
                        modBy (List.length played.run.generations |> max 1) (played.shownGeneration + 1)
                }

        _ ->
            run


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ Backend.automatonRunFinished RunFinished
        , Backend.automatonCensusFinished CensusFinished
        , case model.run of
            Ran _ ->
                Time.every millisecondsBetweenGenerations (\_ -> GenerationInterval)

            _ ->
                Sub.none
        ]


view : Model -> Html Msg
view model =
    div []
        [ div [ Attr.class "controls" ]
            [ CellCountSlider.view largestSeedCellCount CellCountChosen model.cellCount
            , button [ onClick RunRequested ] [ text "Nouvelle graine" ]
            ]
        , viewChoice "Graine"
            SeedKindChosen
            model.seedKind
            [ ( Uniform, "Tirage uniforme" ), ( Growth, "Croissance par voisin" ), ( MostLooped, "La plus bouclée" ) ]
        , viewChoice "Règle"
            RuleChosen
            model.rule
            [ ( Bays4555, "26 voisines : survit à 4–5, naît à 5" )
            , ( Bays5766, "26 voisines : survit à 5–7, naît à 6" )
            , ( Faces2323, "6 voisines : survit à 2–3, naît à 2–3" )
            ]
        , viewRun model.run
        , viewCensus model
        ]


viewChoice : String -> (choice -> Msg) -> choice -> List ( choice, String ) -> Html Msg
viewChoice title onChosen chosen choices =
    let
        viewOption ( choice, label ) =
            button
                [ onClick (onChosen choice)
                , Attr.classList [ ( "tab", True ), ( "shown", choice == chosen ) ]
                ]
                [ text label ]
    in
    div [ Attr.class "controls" ] (span [] [ text (title ++ " :") ] :: List.map viewOption choices)


viewRun : Run -> Html Msg
viewRun run =
    case run of
        Running ->
            p [ Attr.class "status" ] [ text "Calcul des générations…" ]

        Ran played ->
            let
                living =
                    List.drop played.shownGeneration played.run.generations
                        |> List.head
                        |> Maybe.withDefault []
            in
            div []
                [ p [ Attr.class "explanation" ] [ text (describeFate played.run.fate) ]
                , div [ Attr.class "automaton" ] [ Scene.view living ]
                , p [ Attr.class "status" ]
                    [ text
                        ("Génération "
                            ++ String.fromInt played.shownGeneration
                            ++ " sur "
                            ++ String.fromInt (List.length played.run.generations - 1)
                            ++ " : "
                            ++ String.fromInt (List.length living)
                            ++ " cellules vivantes"
                        )
                    ]
                ]

        RunFailed reason ->
            p [ Attr.class "status error" ] [ text ("Automate impossible : " ++ reason) ]


describeFate : Fate -> String
describeFate fate =
    case fate of
        Dies generation ->
            "Cette graine s'éteint à la génération " ++ String.fromInt generation ++ "."

        Still generation ->
            "Cette graine se fige à la génération " ++ String.fromInt generation ++ " et ne bouge plus."

        Cycles generation period ->
            "Cette graine entre à la génération "
                ++ String.fromInt generation
                ++ " dans un cycle de "
                ++ String.fromInt period
                ++ " générations."

        Travels generation period ->
            "Cette graine devient à la génération "
                ++ String.fromInt generation
                ++ " une forme qui se déplace, identique toutes les "
                ++ String.fromInt period
                ++ " générations."

        Unsettled ->
            "Cette graine ne s'est ni éteinte ni répétée avant la limite : elle grossit ou reste active."


viewCensus : Model -> Html Msg
viewCensus model =
    section []
        [ h2 [] [ text "Destin de tous les polycubes de cette taille" ]
        , if model.cellCount > largestCensusCellCount then
            p [ Attr.class "explanation" ]
                [ text ("Recensement limité à " ++ String.fromInt largestCensusCellCount ++ " cubes : au-delà il prend trop de temps.") ]

          else
            case model.census of
                NotCounted ->
                    button [ onClick CensusRequested ] [ text "Recenser" ]

                Counting ->
                    p [ Attr.class "status" ] [ text "Recensement en cours…" ]

                Counted census ->
                    viewCounts census

                CensusFailed reason ->
                    p [ Attr.class "status error" ] [ text ("Recensement impossible : " ++ reason) ]
        ]


viewCounts : AutomatonCensus -> Html Msg
viewCounts census =
    let
        total =
            census.dying + census.still + census.cycling + census.travelling + census.unsettled |> max 1

        viewCount ( label, polycubeCount ) =
            p [ Attr.class "status" ]
                [ text
                    (label
                        ++ " : "
                        ++ String.fromInt polycubeCount
                        ++ " ("
                        ++ String.fromInt (round (100 * toFloat polycubeCount / toFloat total))
                        ++ " %)"
                    )
                ]
    in
    div []
        (List.map viewCount
            [ ( "S'éteignent", census.dying )
            , ( "Se figent", census.still )
            , ( "Entrent dans un cycle", census.cycling )
            , ( "Se déplacent", census.travelling )
            , ( "Sans conclusion à la limite", census.unsettled )
            ]
        )
