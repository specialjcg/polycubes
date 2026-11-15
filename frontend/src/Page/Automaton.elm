module Page.Automaton exposing (Model, Msg, init, start, subscriptions, update, view)

{-| A polycube that grows generation after generation, in one of four ways.
By aggregation, copies of it wander in from afar and stick where they touch.
By mutation, it bears buds that are itself with one cell moved. By budding,
it bears turned copies of itself against it, which bud in turn. As a game of
life, whole polycubes die of crowding while others are born beside them.
-}

import Backend exposing (AutomatonRun)
import CellCountSlider
import Dict
import Html exposing (Html, button, div, p, span, text)
import Html.Attributes as Attr
import Html.Events exposing (onClick)
import Scene
import Time


{-| Must match `MAX_SEED_CELL_COUNT` of the backend.
-}
largestSeedCellCount : Int
largestSeedCellCount =
    6


initialCellCount : Int
initialCellCount =
    5


{-| Life plays some two hundred generations, the others a few dozen.
-}
millisecondsBetweenGenerations : Mode -> Float
millisecondsBetweenGenerations mode =
    case mode of
        Life ->
            250

        _ ->
            800


{-| Generations the last one stays on screen before the seed comes back.
-}
generationsHeldAtTheEnd : Int
generationsHeldAtTheEnd =
    3


type SeedKind
    = Uniform
    | Growth
    | MostLooped


type Mode
    = Aggregation
    | Life
    | Mutation
    | Budding


type alias Model =
    { cellCount : Int
    , seedKind : SeedKind
    , mode : Mode
    , aggregateCellCount : Int
    , lifeWorldReach : Int
    , run : Run
    , isPlaying : Bool
    }


{-| `elapsedGenerations` runs past the last generation while it is held.
`mode` and `seedCellCount` are those the run was asked with, which tell how
its cells are to be coloured.
-}
type Run
    = Running
    | Ran Played
    | Failed String


type alias Played =
    { run : AutomatonRun
    , elapsedGenerations : Int
    , mode : Mode
    , seedCellCount : Int
    }


type Msg
    = CellCountChosen String
    | SeedKindChosen SeedKind
    | ModeChosen Mode
    | AggregateCellCountChosen Int
    | LifeWorldReachChosen Int
    | RunRequested
    | PlaybackToggled
    | RunFinished (Result String AutomatonRun)
    | GenerationInterval


init : Model
init =
    { cellCount = initialCellCount
    , seedKind = Uniform
    , mode = Aggregation
    , aggregateCellCount = 600
    , lifeWorldReach = 5
    , run = Running
    , isPlaying = True
    }


{-| First run, to request once the backend listens.
-}
start : Model -> ( Model, Cmd Msg )
start model =
    requestRun model


requestRun : Model -> ( Model, Cmd Msg )
requestRun model =
    ( { model | run = Running }
    , Backend.requestAutomatonRun
        { cellCount = model.cellCount
        , seedKind = seedKindName model.seedKind
        , mode = modeName model.mode
        , aggregateCellCount = model.aggregateCellCount
        , lifeWorldReach = model.lifeWorldReach
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


{-| Names of the backend's `GrowthMode`.
-}
modeName : Mode -> String
modeName mode =
    case mode of
        Aggregation ->
            "aggregation"

        Life ->
            "life"

        Mutation ->
            "mutation"

        Budding ->
            "budding"


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        CellCountChosen sliderValue ->
            requestRun
                { model | cellCount = CellCountSlider.parse sliderValue |> Maybe.withDefault model.cellCount }

        SeedKindChosen seedKind ->
            requestRun { model | seedKind = seedKind }

        ModeChosen mode ->
            requestRun { model | mode = mode }

        AggregateCellCountChosen aggregateCellCount ->
            requestRun { model | aggregateCellCount = aggregateCellCount }

        LifeWorldReachChosen lifeWorldReach ->
            requestRun { model | lifeWorldReach = lifeWorldReach }

        RunRequested ->
            requestRun model

        PlaybackToggled ->
            ( { model | isPlaying = not model.isPlaying }, Cmd.none )

        RunFinished (Ok run) ->
            ( { model
                | run =
                    Ran
                        { run = run
                        , elapsedGenerations = 0
                        , mode = model.mode
                        , seedCellCount = model.cellCount
                        }
              }
            , Cmd.none
            )

        RunFinished (Err reason) ->
            ( { model | run = Failed reason }, Cmd.none )

        GenerationInterval ->
            ( { model | run = letAGenerationPass model.run }, Cmd.none )


{-| The run plays in a loop: after the last generation has been held, the
seed comes back.
-}
letAGenerationPass : Run -> Run
letAGenerationPass run =
    case run of
        Ran played ->
            Ran
                { played
                    | elapsedGenerations =
                        modBy (List.length played.run.generations + generationsHeldAtTheEnd)
                            (played.elapsedGenerations + 1)
                }

        _ ->
            run


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ Backend.automatonRunFinished RunFinished
        , case ( model.run, model.isPlaying ) of
            ( Ran _, True ) ->
                Time.every (millisecondsBetweenGenerations model.mode) (\_ -> GenerationInterval)

            _ ->
                Sub.none
        ]


view : Model -> Html Msg
view model =
    div []
        [ div [ Attr.class "controls" ]
            [ CellCountSlider.view largestSeedCellCount CellCountChosen model.cellCount
            , button [ onClick RunRequested ] [ text "Recommencer" ]
            , button [ onClick PlaybackToggled ]
                [ text
                    (if model.isPlaying then
                        "Pause"

                     else
                        "Lecture"
                    )
                ]
            ]
        , viewChoice "Graine"
            SeedKindChosen
            model.seedKind
            [ ( Uniform, "Tirage uniforme" ), ( Growth, "Croissance par voisin" ), ( MostLooped, "La plus bouclée" ) ]
        , viewChoice "Croissance"
            ModeChosen
            model.mode
            [ ( Aggregation, "Agrégation" ), ( Life, "Vie" ), ( Mutation, "Mutation" ), ( Budding, "Bourgeonnement" ) ]
        , case model.mode of
            Aggregation ->
                viewChoice "Taille"
                    AggregateCellCountChosen
                    model.aggregateCellCount
                    [ ( 600, "600 cubes" ), ( 3000, "3 000 cubes (≈ 8 s)" ), ( 10000, "10 000 cubes (≈ 45 s)" ) ]

            Life ->
                viewChoice "Monde"
                    LifeWorldReachChosen
                    model.lifeWorldReach
                    [ ( 5, "11 cases de côté (≈ 6 s)" ), ( 7, "15 cases (≈ 13 s)" ), ( 10, "21 cases (≈ 45 s)" ) ]

            _ ->
                text ""
        , p [ Attr.class "explanation" ] [ text (explain model.mode) ]
        , viewRun model.run
        ]


explain : Mode -> String
explain mode =
    case mode of
        Aggregation ->
            "Comme le givre ou le corail : des copies du polycube, tournées au hasard, arrivent de loin en errant case par case et se collent là où elles touchent l'amas pour la première fois. Les pointes attrapent les arrivants avant les creux, d'où les branches. Les couleurs suivent l'ordre d'arrivée : rouge pour les premiers, violet pour les derniers."

        Life ->
            "Un jeu de la vie dont la cellule est un polycube entier. À chaque génération, tous en même temps : un polycube meurt si plus de deux autres le touchent ; un polycube sur deux fait naître un enfant là où il touchera un ou deux vivants. L'enfant a la forme du parent, sauf une fois sur cinq où un cube est déplacé. Une couleur par forme : on voit lesquelles envahissent le monde, un cube dont on choisit le côté, et lesquelles disparaissent."

        Mutation ->
            "À chaque génération, un polycube sur deux tente de bourgeonner : l'enfant est le parent avec un cube déplacé, tourné au hasard, et naît contre lui sans toucher aucun autre. Les formes dérivent le long des branches ; chaque enfant prend une couleur proche de celle de son parent."

        Budding ->
            "À chaque génération, un polycube sur deux tente de bourgeonner : une copie de lui-même, tournée au hasard mais jamais en miroir, naît contre lui là où la place est libre. Chaque copie a sa couleur et bourgeonne à son tour."


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
                lastGeneration =
                    List.length played.run.generations - 1

                shownGeneration =
                    min played.elapsedGenerations lastGeneration

                living =
                    at shownGeneration played.run.generations

                frame =
                    { lowestCorner = played.run.lowestCorner, highestCorner = played.run.highestCorner }
            in
            div []
                [ div [ Attr.class "automaton" ]
                    [ Scene.viewColouredFramed (colouringOf played shownGeneration) frame living ]
                , p [ Attr.class "status" ]
                    [ text
                        ("Génération "
                            ++ String.fromInt shownGeneration
                            ++ " sur "
                            ++ String.fromInt lastGeneration
                            ++ " : "
                            ++ String.fromInt (List.length living // played.seedCellCount)
                            ++ " polycubes, "
                            ++ String.fromInt (List.length living)
                            ++ " cellules"
                            ++ describeShapes (at shownGeneration played.run.shapesByGeneration)
                        )
                    ]
                ]

        Failed reason ->
            p [ Attr.class "status error" ] [ text ("Croissance impossible : " ++ reason) ]


{-| How many shapes live, and how much room the commonest takes: empty when
shapes are not told.
-}
describeShapes : List Int -> String
describeShapes shapes =
    let
        countByShape =
            List.foldl (\shape -> Dict.update shape (Maybe.withDefault 0 >> (+) 1 >> Just)) Dict.empty shapes
    in
    case List.maximum (Dict.values countByShape) of
        Just commonestCount ->
            " ; "
                ++ String.fromInt (Dict.size countByShape)
                ++ " formes, la plus répandue : "
                ++ String.fromInt (100 * commonestCount // List.length shapes)
                ++ " %"

        Nothing ->
            ""


{-| The list at `index`, empty when there is none.
-}
at : Int -> List (List element) -> List element
at index lists =
    List.drop index lists |> List.head |> Maybe.withDefault []


{-| An aggregate lists its cells in the order they came, so they are
coloured by age. Life tells its polycubes apart generation by generation, so
that each stays in place while others die, and colours them by shape. Mutants take after their
parent. Buds are all alike, one colour each.
-}
colouringOf : Played -> Int -> Scene.Colouring
colouringOf played shownGeneration =
    case played.mode of
        Aggregation ->
            Scene.ByAge (List.map List.length played.run.generations |> List.maximum |> Maybe.withDefault 1)

        Life ->
            Scene.ByIdentity played.seedCellCount
                (at shownGeneration played.run.identitiesByGeneration)
                (at shownGeneration played.run.shapesByGeneration)

        Mutation ->
            Scene.ByLineage played.seedCellCount played.run.parentOfCopy

        Budding ->
            Scene.ByCopy played.seedCellCount
