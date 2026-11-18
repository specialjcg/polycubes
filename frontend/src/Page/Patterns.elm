module Page.Patterns exposing (Model, Msg, init, start, subscriptions, update, view)

{-| A gallery of patterns of the Conway-like life of whole polycubes: still
ones, oscillators and gliders, all in relief, found once by
`examples/pattern_gallery.rs`. Each is drawn still; a click plays it in
large under its own rule.
-}

import Backend exposing (AutomatonRun, Pattern, PatternRule)
import Html exposing (Html, div, p, section, text)
import Html.Attributes as Attr
import Html.Events exposing (onClick)
import Polycube exposing (Cell)
import Scene
import Time


millisecondsBetweenGenerations : Float
millisecondsBetweenGenerations =
    300


type alias Model =
    { catalogue : Catalogue
    , player : Player
    }


type Catalogue
    = Loading
    | Loaded (List Pattern)
    | Failed String


type Player
    = Idle
    | Running Pattern
    | Playing Played
    | PlayFailed Pattern String


type alias Played =
    { pattern : Pattern
    , run : AutomatonRun
    , elapsedGenerations : Int
    }


type Msg
    = CatalogueFinished (Result String (List Pattern))
    | PatternChosen Pattern
    | RunFinished (Result String AutomatonRun)
    | GenerationInterval


init : Model
init =
    { catalogue = Loading, player = Idle }


{-| The catalogue, to request once the backend listens.
-}
start : Model -> ( Model, Cmd Msg )
start model =
    ( { model | catalogue = Loading }, Backend.requestPatterns )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        CatalogueFinished (Ok patterns) ->
            ( { model | catalogue = Loaded patterns }, Cmd.none )

        CatalogueFinished (Err reason) ->
            ( { model | catalogue = Failed reason }, Cmd.none )

        PatternChosen pattern ->
            ( { model | player = Running pattern }, Backend.requestPatternRun pattern.number )

        RunFinished result ->
            ( { model | player = finishRun result model.player }, Cmd.none )

        GenerationInterval ->
            ( { model | player = letAGenerationPass model.player }, Cmd.none )


finishRun : Result String AutomatonRun -> Player -> Player
finishRun result player =
    case ( player, result ) of
        ( Running pattern, Ok run ) ->
            Playing { pattern = pattern, run = run, elapsedGenerations = 0 }

        ( Running pattern, Err reason ) ->
            PlayFailed pattern reason

        _ ->
            player


{-| The pattern plays in a loop.
-}
letAGenerationPass : Player -> Player
letAGenerationPass player =
    case player of
        Playing played ->
            Playing
                { played
                    | elapsedGenerations =
                        modBy (List.length played.run.generations) (played.elapsedGenerations + 1)
                }

        _ ->
            player


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ Backend.patternsFinished CatalogueFinished
        , Backend.patternRunFinished RunFinished
        , case model.player of
            Playing _ ->
                Time.every millisecondsBetweenGenerations (\_ -> GenerationInterval)

            _ ->
                Sub.none
        ]


view : Model -> Html Msg
view model =
    div []
        [ p [ Attr.class "explanation" ]
            [ text
                ("Des motifs de la vie à la Conway des polycubes, tous en relief : fixes, "
                    ++ "oscillateurs et planeurs, chacun sous sa propre règle, du plus symétrique "
                    ++ "au plus éclaté. Un clic sur une vignette joue le motif en grand."
                )
            ]
        , viewPlayer model.player
        , viewCatalogue model
        ]


viewPlayer : Player -> Html Msg
viewPlayer player =
    case player of
        Idle ->
            text ""

        Running pattern ->
            p [ Attr.class "status" ] [ text ("Calcul du motif n° " ++ String.fromInt pattern.number ++ "…") ]

        PlayFailed pattern reason ->
            p [ Attr.class "status error" ]
                [ text ("Motif n° " ++ String.fromInt pattern.number ++ " : " ++ reason) ]

        Playing played ->
            let
                living =
                    at played.elapsedGenerations played.run.generations

                frame =
                    { lowestCorner = played.run.lowestCorner, highestCorner = played.run.highestCorner }
            in
            div []
                [ div [ Attr.class "automaton" ]
                    [ Scene.viewColouredFramed
                        (Scene.ByIdentity (cellsPerPolycube played.pattern)
                            (at played.elapsedGenerations played.run.identitiesByGeneration)
                            []
                        )
                        frame
                        living
                    ]
                , p [ Attr.class "status" ]
                    [ text
                        (label played.pattern
                            ++ ", règle "
                            ++ ruleName played.pattern.rule
                            ++ " ; génération "
                            ++ String.fromInt played.elapsedGenerations
                            ++ " sur "
                            ++ String.fromInt (List.length played.run.generations - 1)
                        )
                    ]
                ]


viewCatalogue : Model -> Html Msg
viewCatalogue model =
    case model.catalogue of
        Loading ->
            p [ Attr.class "status" ] [ text "Lecture du catalogue…" ]

        Failed reason ->
            p [ Attr.class "status error" ] [ text reason ]

        Loaded patterns ->
            div [ Attr.class "gallery patterns" ]
                (List.map (viewTile (chosenNumber model.player)) patterns)


viewTile : Maybe Int -> Pattern -> Html Msg
viewTile chosen pattern =
    section
        [ onClick (PatternChosen pattern)
        , Attr.classList [ ( "chosen", chosen == Just pattern.number ) ]
        ]
        [ Scene.viewStill (cellsPerPolycube pattern)
            { lowestCorner = { x = 0, y = 0, z = 0 }, highestCorner = highestCorner pattern }
            (List.concat pattern.polycubes)
        , p [ Attr.class "status" ] [ text (label pattern) ]
        ]


chosenNumber : Player -> Maybe Int
chosenNumber player =
    case player of
        Idle ->
            Nothing

        Running pattern ->
            Just pattern.number

        PlayFailed pattern _ ->
            Just pattern.number

        Playing played ->
            Just played.pattern.number


{-| Number, shape, nature and period, as « n° 3 · O4 · planeur, période 12 ».
-}
label : Pattern -> String
label pattern =
    "n° " ++ String.fromInt pattern.number ++ " · " ++ pattern.shape ++ " · " ++ natureName pattern


natureName : Pattern -> String
natureName pattern =
    case pattern.nature of
        "still" ->
            "fixe"

        "oscillator" ->
            "oscillateur, période " ++ String.fromInt pattern.period

        "glider" ->
            "planeur, période " ++ String.fromInt pattern.period

        other ->
            other


{-| As « S2-4 B3-3 F4 »: neighbours to survive, to be born, and faces a
newborn shares at least.
-}
ruleName : PatternRule -> String
ruleName rule =
    "S"
        ++ String.fromInt rule.fewestToSurvive
        ++ "-"
        ++ String.fromInt rule.mostToSurvive
        ++ " B"
        ++ String.fromInt rule.fewestToBeBorn
        ++ "-"
        ++ String.fromInt rule.mostToBeBorn
        ++ " F"
        ++ String.fromInt rule.fewestFacesShared


cellsPerPolycube : Pattern -> Int
cellsPerPolycube pattern =
    List.head pattern.polycubes |> Maybe.map List.length |> Maybe.withDefault 1


highestCorner : Pattern -> Cell
highestCorner pattern =
    let
        cells =
            List.concat pattern.polycubes

        highestAlong coordinate =
            List.map coordinate cells |> List.maximum |> Maybe.withDefault 0
    in
    { x = highestAlong .x, y = highestAlong .y, z = highestAlong .z }


at : Int -> List (List element) -> List element
at index lists =
    List.drop index lists |> List.head |> Maybe.withDefault []
