#include "shim.h"

#include <QtCore/QAbstractItemModel>
#include <QtCore/QCoreApplication>
#include <QtCore/QDir>
#include <QtCore/QLocale>
#include <QtCore/QStandardPaths>
#include <QtCore/QTranslator>
#include <QtGui/QGuiApplication>
#include <QtGui/QImage>
#include <QtQuick/QQuickItem>
#include <QtQuick/QQuickWindow>
#ifdef QBT_TEST_HOOKS
#include <QtTest/QAbstractItemModelTester>
#endif

bool qbt_grab_window(const QString& path)
{
    const auto windows = QGuiApplication::allWindows();
    for (QWindow* w : windows) {
        if (auto* q = qobject_cast<QQuickWindow*>(w)) {
            return q->grabWindow().save(path);
        }
    }
    return false;
}

bool qbt_install_translator(const QString& extra_dir)
{
    // QBREMOTE_LOCALE (e.g. "de_DE") overrides the system locale.
    const QString override_locale = qEnvironmentVariable("QBREMOTE_LOCALE");
    const QString locale = override_locale.isEmpty() ? QLocale::system().name() : override_locale;
    QStringList dirs;
    if (!extra_dir.isEmpty()) dirs << extra_dir;
    dirs << QCoreApplication::applicationDirPath() + QStringLiteral("/translations");
    for (const QString& base : QStandardPaths::standardLocations(QStandardPaths::AppDataLocation))
        dirs << base + QStringLiteral("/translations");
    // "de_DE" falls back to "de".
    const QStringList names{locale, locale.section(QLatin1Char('_'), 0, 0)};
    for (const QString& dir : std::as_const(dirs)) {
        for (const QString& n : names) {
            auto* t = new QTranslator(QCoreApplication::instance());
            if (t->load(QStringLiteral("qbremote_") + n, dir)) {
                return QCoreApplication::installTranslator(t);
            }
            delete t;
        }
    }
    return false;
}

int qbt_attach_model_testers()
{
#ifndef QBT_TEST_HOOKS
    return 0; // Qt Test is only linked into builds with the `test-hooks` feature.
#else
    int attached = 0;
    const auto windows = QGuiApplication::allWindows();
    for (QWindow* w : windows) {
        auto* q = qobject_cast<QQuickWindow*>(w);
        if (!q) continue;
        QList<QAbstractItemModel*> models = q->findChildren<QAbstractItemModel*>();
        if (q->contentItem()) models += q->contentItem()->findChildren<QAbstractItemModel*>();
        for (QAbstractItemModel* m : std::as_const(models)) {
            if (m->property("qbtTested").toBool()) continue;
            m->setProperty("qbtTested", true);
            // Fatal: an inconsistent begin/end sequence aborts the process, failing the test run.
            new QAbstractItemModelTester(m, QAbstractItemModelTester::FailureReportingMode::Fatal, m);
            ++attached;
        }
    }
    return attached;
#endif
}
