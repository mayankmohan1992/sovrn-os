import QtQuick 2.0
import calamares.slideshow 1.0

Presentation
{
    id: presentation

    Timer {
        interval: 4000
        running: true
        repeat: true
        onTriggered: presentation.goToNextSlide()
    }

    Slide {
        Image {
            id: logo
            source: "logo.png"
            anchors.centerIn: parent
            width: 128
            height: 128
        }
        Text {
            anchors.top: logo.bottom
            anchors.topMargin: 16
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Welcome to Sovrn OS"
            font.pointSize: 24
            color: "#F9FAFB"
        }
        Text {
            anchors.top: parent.top
            anchors.topMargin: 320
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Your decentralized mesh operating system"
            font.pointSize: 14
            color: "#9CA3AF"
        }
    }

    Slide {
        Text {
            anchors.centerIn: parent
            text: "Sovrn OS — Own Your Network"
            font.pointSize: 20
            color: "#F9FAFB"
        }
        Text {
            anchors.top: parent.top
            anchors.topMargin: 280
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Mesh networking • Self-sovereign identity • Decentralized apps"
            font.pointSize: 12
            color: "#9CA3AF"
        }
    }
}
