module Page.ClosedForm exposing (Model, Msg, init, start, subscriptions, update, view)

{-| The closed formula against a plain visit. A polycube met by the
enumeration is shown with the cells waiting to extend it. Its extensions by
one to four cubes are counted twice: the visit places them one by one, the
formula only reads the waiting cells and their surroundings. The page runs
through some visited extensions and sets both counts side by side.
-}

import Array
import Backend exposing (ClosedFormComparison, ExtensionSize)
import CellCountSlider
import Html exposing (Html, button, div, li, p, section, table, td, text, th, tr, ul)
import Html.Attributes as Attr
import Html.Events exposing (onClick)
import Polycube exposing (Cell)
import Scene
import Time


millisecondsBetweenExtensions : Float
millisecondsBetweenExtensions =
    600


{-| Must match `MAX_EXTENDED_CELL_COUNT` of the backend.
-}
largestExtendedCellCount : Int
largestExtendedCellCount =
    6


initialCellCount : Int
initialCellCount =
    4


{-| Cubes the closed formula adds at most; the scene is framed for a little
less, since few extensions stretch straight away from the polycube.
-}
largestAddedCellCount : Int
largestAddedCellCount =
    4


framedAddedCellCount : Int
framedAddedCellCount =
    2


type alias Model =
    { cellCount : Int
    , comparison : Comparison
    }


type Comparison
    = Loading
    | Comparing Showing
    | Failed String


type alias Showing =
    { polycube : ClosedFormComparison
    , addedCellCount : Int
    , shownExampleIndex : Int
    }


type Msg
    = CellCountChosen String
    | ComparisonRequested
    | ComparisonFinished (Result String ClosedFormComparison)
    | AddedCellCountChosen Int
    | ExtensionInterval


init : Model
init =
    { cellCount = initialCellCount, comparison = Loading }


{-| First comparison, to request once the backend listens.
-}
start : Model -> ( Model, Cmd Msg )
start model =
    requestComparison model.cellCount


requestComparison : Int -> ( Model, Cmd Msg )
requestComparison cellCount =
    ( { cellCount = cellCount, comparison = Loading }
    , Backend.requestClosedFormComparison cellCount
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        CellCountChosen sliderValue ->
            requestComparison (CellCountSlider.parse sliderValue |> Maybe.withDefault model.cellCount)

        ComparisonRequested ->
            requestComparison model.cellCount

        ComparisonFinished (Ok polycube) ->
            ( { model
                | comparison =
                    Comparing { polycube = polycube, addedCellCount = largestAddedCellCount, shownExampleIndex = 0 }
              }
            , Cmd.none
            )

        ComparisonFinished (Err reason) ->
            ( { model | comparison = Failed reason }, Cmd.none )

        AddedCellCountChosen addedCellCount ->
            ( changeShowing (\showing -> { showing | addedCellCount = addedCellCount, shownExampleIndex = 0 }) model
            , Cmd.none
            )

        ExtensionInterval ->
            ( changeShowing showNextExample model, Cmd.none )


changeShowing : (Showing -> Showing) -> Model -> Model
changeShowing change model =
    case model.comparison of
        Comparing showing ->
            { model | comparison = Comparing (change showing) }

        Loading ->
            model

        Failed _ ->
            model


{-| Examples loop: the visit they stand for has no end worth waiting for.
-}
showNextExample : Showing -> Showing
showNextExample showing =
    let
        exampleCount =
            shownExtensionSize showing
                |> Maybe.map (.examples >> List.length)
                |> Maybe.withDefault 0
    in
    { showing | shownExampleIndex = modBy (max exampleCount 1) (showing.shownExampleIndex + 1) }


shownExtensionSize : Showing -> Maybe ExtensionSize
shownExtensionSize showing =
    showing.polycube.extensionSizes
        |> List.drop (showing.addedCellCount - 1)
        |> List.head


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ Backend.closedFormComparisonFinished ComparisonFinished
        , case model.comparison of
            Comparing _ ->
                Time.every millisecondsBetweenExtensions (\_ -> ExtensionInterval)

            Loading ->
                Sub.none

            Failed _ ->
                Sub.none
        ]


view : Model -> Html Msg
view model =
    div []
        [ div [ Attr.class "controls" ]
            [ CellCountSlider.view largestExtendedCellCount CellCountChosen model.cellCount
            , button [ onClick ComparisonRequested ] [ text "Autre polycube" ]
            ]
        , p [ Attr.class "explanation" ]
            [ text "Combien de façons d'ajouter 1 à 4 cubes à ce polycube ? La visite les pose une par une. La formule lit seulement les cases en attente et leur voisinage, sans poser un cube." ]
        , viewComparison model.cellCount model.comparison
        ]


viewComparison : Int -> Comparison -> Html Msg
viewComparison cellCount comparison =
    case comparison of
        Loading ->
            p [ Attr.class "status" ] [ text "Comptage en cours…" ]

        Comparing showing ->
            case shownExtensionSize showing of
                Just extensionSize ->
                    viewShowing cellCount showing extensionSize

                Nothing ->
                    p [ Attr.class "status" ] [ text "Aucune extension à montrer." ]

        Failed reason ->
            p [ Attr.class "status error" ] [ text ("Comptage impossible : " ++ reason) ]


viewShowing : Int -> Showing -> ExtensionSize -> Html Msg
viewShowing cellCount showing extensionSize =
    let
        shownExample =
            Array.fromList extensionSize.examples
                |> Array.get showing.shownExampleIndex
                |> Maybe.withDefault []

        waitingCells =
            List.filter (\cell -> not (List.member cell shownExample)) showing.polycube.candidates
    in
    div [ Attr.class "closed-form" ]
        [ section []
            [ viewLegend showing.polycube shownExample
            , Scene.viewAmongNeighbours
                { largestCellCount = cellCount + framedAddedCellCount
                , candidates = waitingCells
                , discarded = []
                , added = shownExample
                }
                showing.polycube.placed
            , p [ Attr.class "status" ] [ text (describeExample showing extensionSize) ]
            ]
        , section []
            [ div [ Attr.class "controls" ] (List.map (viewAddedCellCountButton showing.addedCellCount) (List.range 1 largestAddedCellCount))
            , viewCounts extensionSize
            ]
        ]


viewLegend : ClosedFormComparison -> List Cell -> Html Msg
viewLegend polycube shownExample =
    ul [ Attr.class "legend" ]
        [ li [ Attr.class "placed" ] [ text (String.fromInt (List.length polycube.placed) ++ " cubes du polycube") ]
        , li [ Attr.class "candidate" ] [ text (String.fromInt (List.length polycube.candidates) ++ " cases en attente") ]
        , li [ Attr.class "added" ] [ text (String.fromInt (List.length shownExample) ++ " cubes ajoutés") ]
        ]


describeExample : Showing -> ExtensionSize -> String
describeExample showing extensionSize =
    "Extension montrée : "
        ++ String.fromInt (showing.shownExampleIndex + 1)
        ++ " sur "
        ++ String.fromInt (List.length extensionSize.examples)
        ++ " exemples, pris parmi les "
        ++ String.fromInt (List.sum extensionSize.visitedCounts)
        ++ " visitées"


viewAddedCellCountButton : Int -> Int -> Html Msg
viewAddedCellCountButton shownAddedCellCount addedCellCount =
    button
        [ onClick (AddedCellCountChosen addedCellCount)
        , Attr.classList [ ( "tab", True ), ( "shown", addedCellCount == shownAddedCellCount ) ]
        ]
        [ text ("+" ++ String.fromInt addedCellCount) ]


{-| Both counts, split by how many added cubes lie beyond the waiting cells:
the formula has one term per row.
-}
viewCounts : ExtensionSize -> Html Msg
viewCounts extensionSize =
    let
        viewRow rowAttributes title formulaCount visitedCount =
            tr rowAttributes
                [ th [] [ text title ]
                , td [] [ text (String.fromInt formulaCount) ]
                , td [] [ text (String.fromInt visitedCount) ]
                ]

        termRows =
            List.map3 (\cellsBeyond -> viewRow [] (describeTerm cellsBeyond))
                (List.range 0 (List.length extensionSize.formulaCounts - 1))
                extensionSize.formulaCounts
                extensionSize.visitedCounts
    in
    table [ Attr.class "counts" ]
        (tr [] [ th [] [ text "Cubes ajoutés" ], th [] [ text "Formule" ], th [] [ text "Visite" ] ]
            :: termRows
            ++ [ viewRow [ Attr.class "total" ]
                    "Total"
                    (List.sum extensionSize.formulaCounts)
                    (List.sum extensionSize.visitedCounts)
               ]
        )


describeTerm : Int -> String
describeTerm cellsBeyond =
    case cellsBeyond of
        0 ->
            "tous sur des cases en attente"

        1 ->
            "dont 1 au-delà"

        _ ->
            "dont " ++ String.fromInt cellsBeyond ++ " au-delà"
