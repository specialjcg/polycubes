module Page.Automaton exposing (Model, Msg, init, start, subscriptions, update, view)

{-| A polycube that grows generation after generation, in one of seven ways.
By aggregation, copies of it wander in from afar and stick where they touch.
By mutation, it bears buds that are itself with one cell moved.
By budding, it bears turned copies of itself against it, which bud in turn.
As a replicator, under Fredkin's parity rule, it comes back as six copies of
itself at each power of two. As a colony, its cells divide at random into
the free cells beside them.
-}

import Backend exposing (AutomatonRun)
import CellCountSlider
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


millisecondsBetweenGenerations : Float
millisecondsBetweenGenerations =
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
    | Mutation
    | Budding
    | Life
    | Replicator
    | EdgeReplicator
    | Colony


type alias Model =
    { cellCount : Int
    , seedKind : SeedKind
    , mode : Mode
    , run : Run
    , isPlaying : Bool
    }


{-| `elapsedGenerations` runs past the last generation while it is held.
-}
type Run
    = Running
    | Ran { run : AutomatonRun, elapsedGenerations : Int, colouring : Scene.Colouring }
    | Failed String


type Msg
    = CellCountChosen String
    | SeedKindChosen SeedKind
    | ModeChosen Mode
    | RunRequested
    | PlaybackToggled
    | RunFinished (Result String AutomatonRun)
    | GenerationInterval


init : Model
init =
    { cellCount = initialCellCount
    , seedKind = Uniform
    , mode = Aggregation
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

        Mutation ->
            "mutation"

        Budding ->
            "budding"

        Life ->
            "life"

        Replicator ->
            "replicator"

        EdgeReplicator ->
            "edgeReplicator"

        Colony ->
            "colony"


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
                        , colouring = colouringOf model.mode model.cellCount run
                        }
              }
            , Cmd.none
            )

        RunFinished (Err reason) ->
            ( { model | run = Failed reason }, Cmd.none )

        GenerationInterval ->
            ( { model | run = letAGenerationPass model.run }, Cmd.none )


{-| Budding keeps the seed whole, one colour per copy. A colony lists its
cells in birth order, so they are coloured by age. The replicators have
neither, their cells are coloured by where they stand.
-}
colouringOf : Mode -> Int -> AutomatonRun -> Scene.Colouring
colouringOf mode seedCellCount run =
    case mode of
        Mutation ->
            Scene.ByLineage seedCellCount run.parentOfCopy

        Budding ->
            Scene.ByCopy seedCellCount

        Aggregation ->
            Scene.ByAge (List.map List.length run.generations |> List.maximum |> Maybe.withDefault 1)

        Colony ->
            Scene.ByAge (List.map List.length run.generations |> List.maximum |> Maybe.withDefault 1)

        Life ->
            Scene.ByDistance

        Replicator ->
            Scene.ByDistance

        EdgeReplicator ->
            Scene.ByDistance


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
                Time.every millisecondsBetweenGenerations (\_ -> GenerationInterval)

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
            [ ( Aggregation, "Agrégation" ), ( Mutation, "Mutation" ), ( Budding, "Bourgeonnement" ), ( Life, "Vie" ), ( Replicator, "Réplicateur à 6" ), ( EdgeReplicator, "Réplicateur à 12" ), ( Colony, "Colonie" ) ]
        , p [ Attr.class "explanation" ] [ text (explain model.mode) ]
        , viewRun model.run
        ]


explain : Mode -> String
explain mode =
    case mode of
        Aggregation ->
            "Comme le givre ou le corail : des copies du polycube, tournées au hasard, arrivent de loin en errant case par case et se collent là où elles touchent l'amas pour la première fois. Les pointes attrapent les arrivants avant les creux, d'où les branches. Les couleurs suivent l'ordre d'arrivée : rouge pour les premiers, violet pour les derniers."

        Mutation ->
            "À chaque génération, un polycube sur deux tente de bourgeonner : l'enfant est le parent avec un cube déplacé, tourné au hasard, et naît contre lui sans toucher aucun autre. Les formes dérivent le long des branches ; chaque enfant prend une couleur proche de celle de son parent."

        Budding ->
            "À chaque génération, un polycube sur deux tente de bourgeonner : une copie de lui-même, tournée au hasard mais jamais en miroir, naît contre lui là où la place est libre. Chaque copie a sa couleur et bourgeonne à son tour."

        Life ->
            "Règle de la famille du jeu de la vie, sur les 26 voisines : une cellule survit avec 3 à 6 voisines vivantes, une case vide naît avec exactement 5. Aucun hasard dans la règle ; le départ est une soupe d'une vingtaine de cubes, faite de plusieurs polycubes jetés ensemble. Les couleurs suivent la distance au centre."

        Replicator ->
            "Règle de parité de Fredkin : une case vit si un nombre impair de ses six voisines vivent. Quelle que soit la graine, elle réapparaît en six exemplaires aux générations 1, 2, 4 et 8, de plus en plus écartés. Les couleurs suivent la distance au centre."

        EdgeReplicator ->
            "La même règle de parité, mais sur les douze voisines par arête : la graine réapparaît en douze exemplaires aux générations 1, 2 et 4, vers le milieu des arêtes d'un cube. Les couleurs suivent la distance au centre."

        Colony ->
            "À chaque génération, chaque cellule se divise avec 4 chances sur 10, vers une case libre voisine qui ne touche aucune autre cellule : la colonie pousse en branches, sans jamais se refermer en boucle. Les couleurs suivent l'âge : les cellules les plus anciennes sont rouges, les plus jeunes violettes."


describePopulation : Scene.Colouring -> Int -> String
describePopulation colouring cellCount =
    case colouring of
        Scene.ByLineage cellsPerCopy _ ->
            describeCopies cellsPerCopy cellCount

        Scene.ByCopy cellsPerCopy ->
            describeCopies cellsPerCopy cellCount

        _ ->
            String.fromInt cellCount ++ " cellules"


describeCopies : Int -> Int -> String
describeCopies cellsPerCopy cellCount =
    String.fromInt (cellCount // cellsPerCopy) ++ " polycubes, " ++ String.fromInt cellCount ++ " cellules"


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
                    List.drop shownGeneration played.run.generations
                        |> List.head
                        |> Maybe.withDefault []

                frame =
                    { lowestCorner = played.run.lowestCorner, highestCorner = played.run.highestCorner }
            in
            div []
                [ div [ Attr.class "automaton" ]
                    [ Scene.viewColouredFramed played.colouring frame living ]
                , p [ Attr.class "status" ]
                    [ text
                        ("Génération "
                            ++ String.fromInt shownGeneration
                            ++ " sur "
                            ++ String.fromInt lastGeneration
                            ++ " : "
                            ++ describePopulation played.colouring (List.length living)
                        )
                    ]
                ]

        Failed reason ->
            p [ Attr.class "status error" ] [ text ("Croissance impossible : " ++ reason) ]
