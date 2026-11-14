module Page.Automaton exposing (Model, Msg, init, start, subscriptions, update, view)

{-| A polycube that grows generation after generation, in one of two ways.
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
    = Replicator
    | Colony


type alias Model =
    { cellCount : Int
    , seedKind : SeedKind
    , mode : Mode
    , run : Run
    }


{-| `elapsedGenerations` runs past the last generation while it is held.
-}
type Run
    = Running
    | Ran { run : AutomatonRun, elapsedGenerations : Int }
    | Failed String


type Msg
    = CellCountChosen String
    | SeedKindChosen SeedKind
    | ModeChosen Mode
    | RunRequested
    | RunFinished (Result String AutomatonRun)
    | GenerationInterval


init : Model
init =
    { cellCount = initialCellCount
    , seedKind = Uniform
    , mode = Replicator
    , run = Running
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
        Replicator ->
            "replicator"

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

        RunFinished (Ok run) ->
            ( { model | run = Ran { run = run, elapsedGenerations = 0 } }, Cmd.none )

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
            , button [ onClick RunRequested ] [ text "Recommencer" ]
            ]
        , viewChoice "Graine"
            SeedKindChosen
            model.seedKind
            [ ( Uniform, "Tirage uniforme" ), ( Growth, "Croissance par voisin" ), ( MostLooped, "La plus bouclée" ) ]
        , viewChoice "Croissance"
            ModeChosen
            model.mode
            [ ( Replicator, "Réplicateur" ), ( Colony, "Colonie" ) ]
        , p [ Attr.class "explanation" ] [ text (explain model.mode) ]
        , viewRun model.run
        ]


explain : Mode -> String
explain mode =
    case mode of
        Replicator ->
            "Règle de parité de Fredkin : une case vit si un nombre impair de ses six voisines vivent. Quelle que soit la graine, elle réapparaît en six exemplaires aux générations 1, 2, 4 et 8, de plus en plus écartés."

        Colony ->
            "À chaque génération, chaque cellule qui a une case libre à côté d'elle se divise avec 4 chances sur 10, vers une case libre prise au hasard. Les cellules les plus anciennes sont les plus sombres."


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
            in
            div []
                [ div [ Attr.class "automaton" ]
                    [ Scene.viewFramed
                        { lowestCorner = played.run.lowestCorner, highestCorner = played.run.highestCorner }
                        living
                    ]
                , p [ Attr.class "status" ]
                    [ text
                        ("Génération "
                            ++ String.fromInt shownGeneration
                            ++ " sur "
                            ++ String.fromInt lastGeneration
                            ++ " : "
                            ++ String.fromInt (List.length living)
                            ++ " cellules"
                        )
                    ]
                ]

        Failed reason ->
            p [ Attr.class "status error" ] [ text ("Croissance impossible : " ++ reason) ]
