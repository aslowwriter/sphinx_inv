use std::path::PathBuf;

use sphinx_inv::{SphinxInvError, SphinxInventoryReader};

macro_rules! test_remote_object {
    ($link:literal; $name:ident) => {
        #[test]
        fn $name() -> Result<(), SphinxInvError> {
            let path = PathBuf::from("tests/sphinx_objects/")
                .join(stringify!($name))
                .with_extension("inv");
            let reader = SphinxInventoryReader::from_path(path).unwrap();

            let (success, errors): (Vec<_>, Vec<_>) = reader.partition(Result::is_ok);

            let num_succeeded = success.len();
            let num_failed = errors.len();
            let total = num_succeeded + num_succeeded;

            #[allow(clippy::cast_precision_loss)]
            let perc = (num_failed as f32 / total as f32) * 100.0 as f32;

            for err in errors {
                print!("{}", err.unwrap_err());
            }

            println!("{} out of {} failed ({}%)", num_failed, total, perc);

            assert_eq!(num_failed, 0);

            Ok(())
        }
    };
}

//
test_remote_object!(
    "https://bugzilla.readthedocs.io/en/latest/objects.inv";
    bugzilla
);
test_remote_object!("http://anh.cs.luc.edu:80/python/hands-on/3.1/handsonHtml/objects.inv"; anh_python);
test_remote_object!("http://docs.turbulenz.com/objects.inv"; turbulenz);
test_remote_object!("https://6.docs.plone.org/objects.inv"; plone);
test_remote_object!("https://abrt.readthedocs.io/en/latest/objects.inv"; rtd_abrt);
test_remote_object!("https://alabaster.readthedocs.io/en/latest/objects.inv"; rtd_alabaster);
test_remote_object!("https://alembic.sqlalchemy.org/en/latest/objects.inv"; sqlalchemy);
test_remote_object!("https://apt-team.pages.debian.net/python-apt/objects.inv"; debian);
test_remote_object!("https://arblib.org/objects.inv"; arblib);
test_remote_object!("https://aria2.github.io/manual/en/html/objects.inv"; aria2);
test_remote_object!("https://blinker.readthedocs.io/en/stable/objects.inv"; blinker);
test_remote_object!("https://cartopy.readthedocs.io/stable/objects.inv"; cartopy);
test_remote_object!("https://click.palletsprojects.com/en/stable/objects.inv"; click);
test_remote_object!("https://cmake.org/cmake/help/latest/objects.inv"; cmake);
test_remote_object!("https://conda.io/docs/objects.inv"; conda);
test_remote_object!("https://cython.readthedocs.io/en/latest/objects.inv"; cython);
test_remote_object!("https://damask-multiphysics.org/objects.inv"; dmask);
test_remote_object!("https://deap.readthedocs.io/en/master/objects.inv"; deap);
test_remote_object!("https://devguide.python.org/objects.inv"; python_dev_guide);
test_remote_object!("https://discovery.gitlabpages.inria.fr/enoslib/objects.inv"; enoslib);
test_remote_object!("https://djangocas.dev/docs/latest/objects.inv"; djangocas);
test_remote_object!("https://django-q.readthedocs.io/en/latest/objects.inv"; django_q);
test_remote_object!("https://doc.coreboot.org/objects.inv"; coreboot);
test_remote_object!("https://doc.sagemath.org/html/en/installation/objects.inv"; sagemath);
test_remote_object!("https://docs.ansible.com/projects/ansible/latest/objects.inv"; ansible);
test_remote_object!("https://docs.blender.org/api/current/objects.inv"; blender);
test_remote_object!("https://docs.bokeh.org/en/latest/objects.inv"; bokeh);
test_remote_object!("https://docs.buildbot.net/latest/objects.inv"; buildbot);
test_remote_object!("https://docs.cherrypy.dev/en/latest/objects.inv"; cherrpy);
test_remote_object!("https://docs.couchdb.org/en/stable/objects.inv"; couchdb);
test_remote_object!("https://docs.cupy.dev/en/stable/objects.inv"; cupy);
test_remote_object!("https://docs.enthought.com/chaco/objects.inv"; chaco);
test_remote_object!("https://docs.enthought.com/mayavi/mayavi/objects.inv"; mayavi);
test_remote_object!("https://docs.enthought.com/mayavi/tvtk/objects.inv"; tvtk);
test_remote_object!("https://docs.fabfile.org/en/latest/objects.inv"; fablife);
test_remote_object!("https://docs.jupyter.org/en/latest/objects.inv"; jupyter);
test_remote_object!("https://docs.makotemplates.org/objects.inv"; makotemplates);
test_remote_object!("https://docs.mediagoblin.org/en/stable/objects.inv"; mediagoblin);
test_remote_object!("https://docs.mesa3d.org/objects.inv"; mesa3d);
test_remote_object!("https://docs.mongodb.com/objects.inv"; mongodb);
test_remote_object!("https://docs.netgate.com/objects.inv"; netgate);
test_remote_object!("https://docs.nextcloud.com/server/latest/user_manual/en/objects.inv"; nextcloud);
test_remote_object!("https://docs.obspy.org/objects.inv"; obspy);
// test_remote_object!("https://docs.opencv.org/2.4.13.7/objects.inv"; opencv);
test_remote_object!("https://docs.pagure.org/copr.copr/objects.inv"; copr);
test_remote_object!("https://docs.panda3d.org/1.10/objects.inv"; panda3d);
test_remote_object!("https://docs.podman.io/en/latest/objects.inv"; podman);
test_remote_object!("https://docs.pycantonese.org/stable/objects.inv"; pycantonese);
test_remote_object!("https://docs.pyinvoke.org/en/stable/objects.inv"; pyinvoke);
test_remote_object!("https://docs.pylangacq.org/stable/objects.inv"; pylangacq);
test_remote_object!("https://docs.python-eve.org/en/stable/objects.inv"; pyeve);
test_remote_object!("https://docs.python.org/3/objects.inv"; python3);
// test_remote_object!("https://docs.pyvista.org/objects.inv"; pyvista);
test_remote_object!("https://docs.sepal.io/en/latest/objects.inv"; sepal);
test_remote_object!("https://docs.spring.io/spring-python/1.2.x/sphinx/html/objects.inv"; spring);
test_remote_object!("https://docs.spyder-ide.org/current/objects.inv"; spyder);
test_remote_object!("https://docs.sympy.org/latest/objects.inv"; sympy);
test_remote_object!("https://docs.typo3.org/objects.inv"; typo3);
test_remote_object!("https://docs.valence.desire2learn.com/objects.inv"; valence);
test_remote_object!("https://documen.tician.de/codepy/objects.inv"; codepy);
test_remote_object!("https://documen.tician.de/hedge/objects.inv"; hedge);
test_remote_object!("https://documen.tician.de/meshpy/objects.inv"; meshpy);
test_remote_object!("https://documen.tician.de/pycuda/objects.inv"; pycuda);
test_remote_object!("https://documen.tician.de/pyopencl/objects.inv"; pyopencl);
test_remote_object!("https://documen.tician.de/pyublas/objects.inv"; pyublas);
test_remote_object!("https://documen.tician.de/pyvisfile/objects.inv"; pyvisfile);
test_remote_object!("https://easybuild.readthedocs.io/objects.inv"; easybuild);
test_remote_object!("https://fairlearn.org/main/objects.inv"; fairlearn);
test_remote_object!("https://feature-engine.readthedocs.io/en/latest/objects.inv"; feature_engine);
test_remote_object!("https://flask.palletsprojects.com/en/stable/objects.inv"; flask);
test_remote_object!("https://getfem.org/objects.inv"; getfem);
test_remote_object!("https://getmarbl.readthedocs.io/en/latest/objects.inv"; getmarbl);
test_remote_object!("https://guides.dataverse.org/en/latest/objects.inv"; dataverse);
// test_remote_object!("https://jinja.palletsprojects.com/en/stable/objects.inv"; jinja);
test_remote_object!("https://jupyterbook.org/en/stable/objects.inv"; jupyterbook);
test_remote_object!("https://jython.readthedocs.io/en/latest/objects.inv"; jthon);
test_remote_object!("https://leo-editor.github.io/leo-editor/objects.inv"; leo_editor);
test_remote_object!("https://liblas.org/objects.inv"; liblas);
test_remote_object!("https://linguistica-uchicago.github.io/lxa5/objects.inv"; lxa5);
test_remote_object!("https://llvm.org/docs/objects.inv"; llvm);
test_remote_object!("https://manual.calibre-ebook.com/objects.inv"; calire);
test_remote_object!("https://matplotlib.org/stable/objects.inv"; matplotlib);
test_remote_object!("https://mne.tools/stable/objects.inv"; mne);
test_remote_object!("https://momotor.org/doc/lti/objects.inv"; momotor);
test_remote_object!("https://mpastell.com/pweave/objects.inv"; mpastell);
test_remote_object!("https://mpmath.org/doc/current/objects.inv"; mpmath);
test_remote_object!("https://mybinder.readthedocs.io/en/latest/objects.inv"; mybiner);
test_remote_object!("https://networkx.org/documentation/stable/objects.inv"; networkx);
test_remote_object!("https://noble.gs.washington.edu/proj/genomedata/doc/1.3.3/objects.inv"; genomedata);
test_remote_object!("https://noble.gs.washington.edu/proj/segway/doc/1.1.0/objects.inv"; segway);
test_remote_object!("https://numpy.org/doc/stable/objects.inv"; numpy);
test_remote_object!("https://openturns.github.io/openturns/latest/objects.inv"; openturns);
test_remote_object!("https://packaging.python.org/objects.inv"; packaging);
test_remote_object!("https://pageperso.lis-lab.fr/~edouard.thiel/ez-draw/doc/en/html/objects.inv"; pageperso);
test_remote_object!("https://pandas.pydata.org/docs/objects.inv"; pandas);
test_remote_object!("https://pros.cs.purdue.edu/v5/objects.inv"; purdue);
test_remote_object!("https://pyemd.readthedocs.io/en/latest/objects.inv"; pyemd);
// test_remote_object!("https://pygments.org/objects.inv"; pygments);
test_remote_object!("https://pymotw.com/2/objects.inv"; pymotw);
test_remote_object!("https://pyqt.sourceforge.net/Docs/PyQt4/objects.inv"; pyqt4);
test_remote_object!("https://pyqt.sourceforge.net/Docs/PyQt5/objects.inv"; pyqt5);
test_remote_object!("https://pyre.readthedocs.io/en/latest/objects.inv"; pyre);
test_remote_object!("https://pyspace.github.io/pyspace/objects.inv"; pyspace);
test_remote_object!("https://pystra.github.io/pystra/objects.inv"; pystra);
test_remote_object!("https://python.arviz.org/en/stable/objects.inv"; arviz);
test_remote_object!("https://pythonhosted.org/Flask-OpenID/objects.inv"; flask_openid);
test_remote_object!("https://qutip.readthedocs.io/en/latest/objects.inv"; qutip);
test_remote_object!("https://radimrehurek.com/gensim/objects.inv"; radimrehurek);
test_remote_object!("https://renderdoc.org/docs/objects.inv"; renderdoc);
test_remote_object!("https://requests.readthedocs.io/en/latest/objects.inv"; requests);
test_remote_object!("https://ring-lang.github.io/doc1.20/objects.inv"; ring_lang);
test_remote_object!("https://rogerbinns.github.io/apsw/objects.inv"; rogerbinns);
test_remote_object!("https://runawayhorse001.github.io/SphinxGithub/objects.inv"; runawayhorse001);
test_remote_object!("https://ryan-roemer.github.io/sphinx-bootstrap-theme/objects.inv"; ryan_roemer);
test_remote_object!("https://scikit-learn.org/stable/objects.inv"; scikit_learn);
test_remote_object!("https://seaborn.pydata.org/objects.inv"; seaborn);
test_remote_object!("https://searx.github.io/searx/objects.inv"; searx);
test_remote_object!("https://simgrid.org/doc/latest/objects.inv"; simgrid);
test_remote_object!("https://six.readthedocs.io/objects.inv"; six);
test_remote_object!("https://tango-controls.readthedocs.io/projects/pytango/en/latest/objects.inv"; tango);
test_remote_object!("https://tuleap.net/doc/en/objects.inv"; tuleap);
test_remote_object!("https://turbogears.readthedocs.io/en/latest/objects.inv"; turbogears);
test_remote_object!("https://urllib3.readthedocs.io/en/stable/objects.inv"; urllib3);
test_remote_object!("https://vmlaker.github.io/mpipe/objects.inv"; mpipe);
test_remote_object!("https://waf.io/apidocs/objects.inv"; waf);
test_remote_object!("https://wtforms.readthedocs.io/objects.inv"; wtforms);
test_remote_object!("https://www.breezy-vcs.org/doc/en/objects.inv"; breezy);
test_remote_object!("https://www.crummy.com/software/BeautifulSoup/bs4/doc/objects.inv"; crummy);
test_remote_object!("https://www.gevent.org/objects.inv"; gevent);
test_remote_object!("https://www.ixsystems.com/documentation/truenas/11.3-U5/objects.inv"; ixsystems);
test_remote_object!("https://www.kernel.org/doc/html/latest/objects.inv"; linux_kernel_docs);
test_remote_object!("https://www.lino-framework.org/objects.inv"; lino_framework);
test_remote_object!("https://www.mdtraj.org/1.9.8.dev0/objects.inv"; mdtraj);
test_remote_object!("https://www.nltk.org/objects.inv"; nltk);
test_remote_object!("https://www.nongnu.org/gsl-shell/objects.inv"; nongnu);
test_remote_object!("https://www.psicode.org/psi4manual/master/objects.inv"; psicode);
test_remote_object!("https://www.psycopg.org/docs/objects.inv"; psycopg);
test_remote_object!("https://www.pygame.org/docs/objects.inv"; pygame);
test_remote_object!("https://www.pypa.io/en/latest/objects.inv"; pypa);
test_remote_object!("https://www.rdkit.org/docs/objects.inv"; rdkit);
test_remote_object!("https://www.roundup-tracker.org/objects.inv"; roudup_tracker);
test_remote_object!("https://www.sphinx-doc.org/en/master/objects.inv"; sphinx_doc);
test_remote_object!("https://www.statsmodels.org/stable/objects.inv"; statsmodels);
test_remote_object!("https://www.varnish-cache.org/objects.inv"; varnish_cache);
test_remote_object!("https://www.writethedocs.org/objects.inv"; write_the_docs);
test_remote_object!("https://www.zope.dev/objects.inv"; zope);
test_remote_object!("https://wxpython.org/Phoenix/docs/html/objects.inv"; wxpython);
