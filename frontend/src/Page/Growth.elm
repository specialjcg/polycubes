module Page.Growth exposing (Model, Msg, init, start, subscriptions, update, view)

{-| Two polycubes of the same size appear cube by cube, side by side: one
grown by sticking each new cube on a random free neighbour, one drawn
uniformly among all fixed polycubes. The first process favours compact
shapes; the second shows what a typical polycube looks like.
-}

import Backend exposing (GrownPolycube, GrowthComparison)
import CellCountSlider
import Html exposing (Html, button, div, h2, p, section, text)
import Html.Attributes as Attr
import Html.Events exposing (onClick)
import Scene
import Time


millisecondsBetweenCubes : Float
millisecondsBetweenCubes =
    450


type alias Model =
    { cellCount : Int
    , comparison : Comparison
    }


type Comparison
    = Loading
    | Growing { shownCellCount : Int, polycubes : GrowthComparison }
    | Failed String


type Msg
    = CellCountChosen String
    | ComparisonRequested
    | ComparisonFinished (Result String GrowthComparison)
    | ReplayRequested
    | CubeInterval


init : Model
init =
    { cellCount = CellCountSlider.initialCellCount, comparison = Loading }


{-| First comparison, to request once the backend listens.
-}
start : Model -> ( Model, Cmd Msg )
start model =
    requestComparison model.cellCount


requestComparison : Int -> ( Model, Cmd Msg )
requestComparison cellCount =
    ( { cellCount = cellCount, comparison = Loading }
    , Backend.requestGrowthComparison cellCount
    )


firstShownCellCount : Int
firstShownCellCount =
    1


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model.comparison ) of
        ( CellCountChosen sliderValue, _ ) ->
            requestComparison (CellCountSlider.parse sliderValue |> Maybe.withDefault model.cellCount)

        ( ComparisonRequested, _ ) ->
            requestComparison model.cellCount

        ( ComparisonFinished (Ok polycubes), _ ) ->
            ( { model | comparison = Growing { shownCellCount = firstShownCellCount, polycubes = polycubes } }
            , Cmd.none
            )

        ( ComparisonFinished (Err reason), _ ) ->
            ( { model | comparison = Failed reason }, Cmd.none )

        ( ReplayRequested, Growing growth ) ->
            ( { model | comparison = Growing { growth | shownCellCount = firstShownCellCount } }
            , Cmd.none
            )

        ( CubeInterval, Growing growth ) ->
            ( { model | comparison = Growing { growth | shownCellCount = growth.shownCellCount + 1 } }
            , Cmd.none
            )

        ( ReplayRequested, _ ) ->
            ( model, Cmd.none )

        ( CubeInterval, _ ) ->
            ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ Backend.growthComparisonFinished ComparisonFinished
        , if isStillGrowing model then
            Time.every millisecondsBetweenCubes (\_ -> CubeInterval)

          else
            Sub.none
        ]


isStillGrowing : Model -> Bool
isStillGrowing model =
    case model.comparison of
        Growing growth ->
            growth.shownCellCount < model.cellCount

        Loading ->
            False

        Failed _ ->
            False


view : Model -> Html Msg
view model =
    div []
        [ div [ Attr.class "controls" ]
            [ CellCountSlider.view CellCountSlider.largestDrawnCellCount CellCountChosen model.cellCount
            , button [ onClick ComparisonRequested ] [ text "Nouveau tirage" ]
            , button [ onClick ReplayRequested ] [ text "Rejouer" ]
            ]
        , viewComparison model.comparison
        ]


viewComparison : Comparison -> Html Msg
viewComparison comparison =
    case comparison of
        Loading ->
            p [ Attr.class "status" ] [ text "Tirage en cours…" ]

        Growing growth ->
            div [ Attr.class "side-by-side" ]
                [ viewGrownPolycube
                    "Croissance par voisin au hasard"
                    "Chaque cube se colle sur une case libre voisine prise au hasard."
                    growth.shownCellCount
                    growth.polycubes.neighbourGrowth
                , viewGrownPolycube
                    "Tirage uniforme"
                    "Polycube pris au hasard parmi tous ceux de cette taille, montré cube par cube."
                    growth.shownCellCount
                    growth.polycubes.uniformDraw
                ]

        Failed reason ->
            p [ Attr.class "status error" ] [ text ("Tirage impossible : " ++ reason) ]


viewGrownPolycube : String -> String -> Int -> GrownPolycube -> Html Msg
viewGrownPolycube title explanation shownCellCount grownPolycube =
    section []
        [ h2 [] [ text title ]
        , p [ Attr.class "explanation" ] [ text explanation ]
        , Scene.viewFirstCells shownCellCount grownPolycube.cells
        , p [ Attr.class "status" ] [ text (describeBoundingBox grownPolycube) ]
        ]


{-| The fuller the bounding box, the more compact the polycube.
-}
describeBoundingBox : GrownPolycube -> String
describeBoundingBox grownPolycube =
    let
        boxVolume =
            List.product grownPolycube.boundingBox

        filledPercentage =
            if boxVolume == 0 then
                0

            else
                round (100 * toFloat (List.length grownPolycube.cells) / toFloat boxVolume)
    in
    "Boîte englobante "
        ++ String.join " × " (List.map String.fromInt grownPolycube.boundingBox)
        ++ ", remplie à "
        ++ String.fromInt filledPercentage
        ++ " %"
