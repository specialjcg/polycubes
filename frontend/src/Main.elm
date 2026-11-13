module Main exposing (main)

{-| Animated views on polycubes. Each page is a module of `Page`, holding its
own state, messages and view.
-}

import Backend
import Browser
import Html exposing (Html, button, h1, main_, nav, text)
import Html.Attributes as Attr
import Html.Events exposing (onClick)
import Page.Draw
import Page.Growth


type Page
    = GrowthPage
    | DrawPage


type alias Model =
    { shownPage : Page
    , growth : Page.Growth.Model
    , draw : Page.Draw.Model
    }


type Msg
    = BackendReady
    | PageChosen Page
    | GrowthMsg Page.Growth.Msg
    | DrawMsg Page.Draw.Msg


main : Program () Model Msg
main =
    Browser.element
        { init = \_ -> ( init, Cmd.none )
        , update = update
        , view = view
        , subscriptions = subscriptions
        }


init : Model
init =
    { shownPage = GrowthPage
    , growth = Page.Growth.init
    , draw = Page.Draw.init
    }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        BackendReady ->
            let
                ( growth, growthCmd ) =
                    Page.Growth.start model.growth

                ( draw, drawCmd ) =
                    Page.Draw.start model.draw
            in
            ( { model | growth = growth, draw = draw }
            , Cmd.batch [ Cmd.map GrowthMsg growthCmd, Cmd.map DrawMsg drawCmd ]
            )

        PageChosen page ->
            ( { model | shownPage = page }, Cmd.none )

        GrowthMsg growthMsg ->
            let
                ( growth, growthCmd ) =
                    Page.Growth.update growthMsg model.growth
            in
            ( { model | growth = growth }, Cmd.map GrowthMsg growthCmd )

        DrawMsg drawMsg ->
            let
                ( draw, drawCmd ) =
                    Page.Draw.update drawMsg model.draw
            in
            ( { model | draw = draw }, Cmd.map DrawMsg drawCmd )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ Backend.ready (\_ -> BackendReady)
        , Sub.map GrowthMsg (Page.Growth.subscriptions model.growth)
        , Sub.map DrawMsg Page.Draw.subscriptions
        ]


view : Model -> Html Msg
view model =
    main_ []
        [ h1 [] [ text "Polycubes" ]
        , nav []
            [ viewPageButton model.shownPage GrowthPage "Croissance biaisée contre tirage uniforme"
            , viewPageButton model.shownPage DrawPage "Tirage uniforme"
            ]
        , case model.shownPage of
            GrowthPage ->
                Html.map GrowthMsg (Page.Growth.view model.growth)

            DrawPage ->
                Html.map DrawMsg (Page.Draw.view model.draw)
        ]


viewPageButton : Page -> Page -> String -> Html Msg
viewPageButton shownPage page title =
    button
        [ onClick (PageChosen page)
        , Attr.classList [ ( "tab", True ), ( "shown", page == shownPage ) ]
        ]
        [ text title ]
